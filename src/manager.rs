use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongW, GetWindowRect,
    IsIconic, IsWindowVisible, SetForegroundWindow, SetWindowPos,
    GWL_EXSTYLE, GWL_STYLE, MSG,
    WS_CHILD, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_APPWINDOW,
    WS_SIZEBOX,
    SWP_NOACTIVATE, SWP_NOZORDER,
    PeekMessageW, TranslateMessage, DispatchMessageW,
    WM_HOTKEY, PostQuitMessage, PM_REMOVE,
    GetWindowTextW, GetClassNameW,
};
use windows::Win32::Graphics::Gdi::{
    MonitorFromPoint, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
};

use crate::animator::AnimationDriver;
use crate::config::Config;
use crate::hotkeys;
use crate::layout::{self, Rect};

// Compensación del borde invisible de Windows 10/11 (DWM shadow border)
// Cada ventana tiene ~7px de borde invisible en lados y abajo
const INVIS_BORDER: i32 = 7;

pub struct WindowManager {
    pub config: Config,
    pub tiles: Vec<isize>,       // todas las ventanas gestionadas
    pub floating: HashSet<isize>,// excluidas manualmente del tiling
    pub focused_idx: usize,
    pub current_layout: String,
    animator: AnimationDriver,
    last_snapshot: Vec<isize>,   // para detectar cambios reales
}

impl WindowManager {
    pub fn new(config: Config) -> Self {
        let layout = config.layout.clone();
        let anim_ms = config.animation_ms;
        Self {
            config,
            tiles: Vec::new(),
            floating: HashSet::new(),
            focused_idx: 0,
            current_layout: layout,
            animator: AnimationDriver::new(anim_ms),
            last_snapshot: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        hotkeys::register_all(&self.config.modifier.clone());
        self.refresh_and_tile();
        self.message_loop();
    }

    pub fn refresh_and_tile(&mut self) {
        let visible = collect_visible_windows();
        let visible_set: HashSet<isize> = visible.iter().copied().collect();

        for &h in &visible {
            if !self.tiles.contains(&h) {
                self.tiles.push(h);
            }
        }
        self.tiles.retain(|h| visible_set.contains(h));
        self.floating.retain(|h| visible_set.contains(h));

        if self.tiles.is_empty() {
            self.focused_idx = 0;
        } else if self.focused_idx >= self.tiles.len() {
            self.focused_idx = self.tiles.len() - 1;
        }

        self.last_snapshot = self.tiles.clone();
        self.retile();
    }

    fn retile(&mut self) {
        let screen = work_area();

        // Separar: ventanas que no tienen WS_SIZEBOX (no redimensionables) flotan
        // Las que sí son redimensionables entran al tiling
        let mut tileable: Vec<isize> = Vec::new();
        let mut fixed: Vec<isize> = Vec::new();

        for &h in &self.tiles {
            if self.floating.contains(&h) { continue; }
            if is_resizable(h) {
                tileable.push(h);
            } else {
                fixed.push(h); // ventanas fijas/diálogos — no tocar
            }
        }

        if tileable.is_empty() { return; }

        let rects = layout::compute(
            &self.current_layout.clone(),
            screen,
            tileable.len(),
            self.config.gap,
            self.config.main_ratio,
        );

        for (hwnd, target) in tileable.iter().zip(rects.iter()) {
            let from = window_rect(*hwnd);
            self.animator.push(*hwnd, from, *target);
        }

        self.drive_animation();
    }

    fn drive_animation(&mut self) {
        if !self.animator.has_active() { return; }

        let snapshots: Vec<(isize, Rect, Rect)> = self.animator.active.iter()
            .map(|a| (a.hwnd, a.from, a.to))
            .collect();
        let duration_ms = self.config.animation_ms;

        if duration_ms == 0 {
            for (hwnd, _, to) in &snapshots {
                apply_rect(*hwnd, *to);
            }
            self.animator.active.clear();
            return;
        }

        thread::spawn(move || {
            let mut driver = AnimationDriver::new(duration_ms);
            for (hwnd, from, to) in snapshots {
                driver.push(hwnd, from, to);
            }
            while driver.has_active() {
                for (hwnd, rect, _) in driver.tick() {
                    apply_rect(hwnd, rect);
                }
                thread::sleep(Duration::from_millis(8));
            }
        });

        self.animator.active.clear();
    }

    pub fn focus_next(&mut self) {
        if self.tiles.is_empty() { return; }
        self.focused_idx = (self.focused_idx + 1) % self.tiles.len();
        self.focus_current();
    }

    pub fn focus_prev(&mut self) {
        if self.tiles.is_empty() { return; }
        if self.focused_idx == 0 { self.focused_idx = self.tiles.len() - 1; }
        else { self.focused_idx -= 1; }
        self.focus_current();
    }

    fn focus_current(&self) {
        if let Some(&hwnd) = self.tiles.get(self.focused_idx) {
            unsafe { let _ = SetForegroundWindow(HWND(hwnd as *mut _)); }
        }
    }

    pub fn swap_next(&mut self) {
        let n = self.tiles.len();
        if n < 2 { return; }
        let next = (self.focused_idx + 1) % n;
        self.tiles.swap(self.focused_idx, next);
        self.focused_idx = next;
        self.retile();
    }

    pub fn swap_prev(&mut self) {
        let n = self.tiles.len();
        if n < 2 { return; }
        let prev = if self.focused_idx == 0 { n - 1 } else { self.focused_idx - 1 };
        self.tiles.swap(self.focused_idx, prev);
        self.focused_idx = prev;
        self.retile();
    }

    pub fn toggle_float(&mut self) {
        if let Some(&hwnd) = self.tiles.get(self.focused_idx) {
            if self.floating.contains(&hwnd) { self.floating.remove(&hwnd); }
            else { self.floating.insert(hwnd); }
            self.retile();
        }
    }

    pub fn set_layout(&mut self, name: &str) {
        self.current_layout = name.to_string();
        self.retile();
    }

    fn message_loop(&mut self) {
        let mut tick: u32 = 0;

        loop {
            unsafe {
                let mut msg = MSG::default();
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == 0x0012 { return; } // WM_QUIT
                    if msg.message == WM_HOTKEY {
                        self.handle_hotkey(msg.wParam.0 as i32);
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            tick += 1;
            // Cada 500ms (50 ticks * 10ms) revisar si cambió algo
            if tick % 50 == 0 {
                let current = collect_visible_windows();
                // Solo retilear si la lista de ventanas cambió de verdad
                if !lists_equal(&current, &self.last_snapshot) {
                    self.last_snapshot = current.clone();
                    let visible_set: HashSet<isize> = current.iter().copied().collect();
                    for &h in &current {
                        if !self.tiles.contains(&h) { self.tiles.push(h); }
                    }
                    self.tiles.retain(|h| visible_set.contains(h));
                    self.floating.retain(|h| visible_set.contains(h));
                    if self.tiles.is_empty() { self.focused_idx = 0; }
                    else if self.focused_idx >= self.tiles.len() {
                        self.focused_idx = self.tiles.len() - 1;
                    }
                    self.retile();
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    fn handle_hotkey(&mut self, id: i32) {
        use hotkeys::id::*;
        match id {
            FOCUS_PREV   => self.focus_prev(),
            FOCUS_NEXT   => self.focus_next(),
            SWAP_NEXT    => self.swap_next(),
            SWAP_PREV    => self.swap_prev(),
            TOGGLE_FLOAT => self.toggle_float(),
            LAYOUT_TALL  => self.set_layout("tall"),
            LAYOUT_WIDE  => self.set_layout("wide"),
            LAYOUT_BSP   => self.set_layout("bsp"),
            LAYOUT_MONO  => self.set_layout("monocle"),
            RETILE       => self.refresh_and_tile(),
            QUIT         => unsafe { PostQuitMessage(0) },
            _            => {}
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn lists_equal(a: &[isize], b: &[isize]) -> bool {
    let mut sa: Vec<isize> = a.to_vec(); sa.sort();
    let mut sb: Vec<isize> = b.to_vec(); sb.sort();
    sa == sb
}

/// Una ventana es redimensionable si tiene WS_SIZEBOX (también llamado WS_THICKFRAME)
fn is_resizable(hwnd: isize) -> bool {
    unsafe {
        let style = GetWindowLongW(HWND(hwnd as *mut _), GWL_STYLE) as u32;
        style & WS_SIZEBOX.0 != 0
    }
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let list = &mut *(lparam.0 as *mut Vec<isize>);
    if is_manageable(hwnd) {
        list.push(hwnd.0 as isize);
    }
    BOOL(1)
}

fn collect_visible_windows() -> Vec<isize> {
    let mut list: Vec<isize> = Vec::new();
    unsafe {
        let ptr = &mut list as *mut Vec<isize> as isize;
        let _ = EnumWindows(Some(enum_proc), LPARAM(ptr));
    }
    list
}

fn is_manageable(hwnd: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() { return false; }
        if IsIconic(hwnd).as_bool() { return false; }

        let style    = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

        if style & WS_CHILD.0 != 0 { return false; }
        if ex_style & WS_EX_TOOLWINDOW.0 != 0
            && ex_style & WS_EX_APPWINDOW.0 == 0 { return false; }
        if ex_style & WS_EX_NOACTIVATE.0 != 0 { return false; }

        let mut title = [0u16; 256];
        if GetWindowTextW(hwnd, &mut title) == 0 { return false; }

        let mut class = [0u16; 256];
        GetClassNameW(hwnd, &mut class);
        let class_str = String::from_utf16_lossy(&class);
        let class_str = class_str.trim_matches('\0');

        let skip = [
            "Shell_TrayWnd", "Progman", "WorkerW", "DV2ControlHost",
            "MsgrIMEWindowClass", "SysShadow", "Button",
            "Windows.UI.Core.CoreWindow", "ApplicationFrameWindow",
            "Shell_SecondaryTrayWnd", "tooltips_class32",
        ];
        if skip.iter().any(|c| class_str == *c) { return false; }

        true
    }
}

fn window_rect(hwnd: isize) -> Rect {
    unsafe {
        let mut r = RECT::default();
        let _ = GetWindowRect(HWND(hwnd as *mut _), &mut r);
        Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top)
    }
}

/// Aplica rect compensando el borde invisible de Win10 (DWM shadow)
fn apply_rect(hwnd: isize, rect: Rect) {
    unsafe {
        let _ = SetWindowPos(
            HWND(hwnd as *mut _), None,
            rect.x - INVIS_BORDER,
            rect.y,
            rect.w + INVIS_BORDER * 2,
            rect.h + INVIS_BORDER,
            SWP_NOACTIVATE | SWP_NOZORDER,
        );
    }
}

fn work_area() -> Rect {
    unsafe {
        use windows::Win32::Foundation::POINT;
        let monitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        let r = info.rcWork;
        Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top)
    }
}
