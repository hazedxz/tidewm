use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, HOT_KEY_MODIFIERS,
    MOD_ALT, MOD_WIN, MOD_CONTROL, MOD_SHIFT,
};
use windows::Win32::Foundation::HWND;

pub const VK_RETURN: u32 = 0x0D;
pub const VK_Q:     u32 = 0x51;
pub const VK_H:     u32 = 0x48;
pub const VK_L:     u32 = 0x4C;
pub const VK_J:     u32 = 0x4A;
pub const VK_K:     u32 = 0x4B;
pub const VK_SPACE: u32 = 0x20;
pub const VK_1:     u32 = 0x31;
pub const VK_2:     u32 = 0x32;
pub const VK_3:     u32 = 0x33;
pub const VK_4:     u32 = 0x34;

pub mod id {
    pub const FOCUS_PREV:    i32 = 1;
    pub const FOCUS_NEXT:    i32 = 2;
    pub const SWAP_NEXT:     i32 = 5;
    pub const SWAP_PREV:     i32 = 6;
    pub const TOGGLE_FLOAT:  i32 = 7;
    pub const LAYOUT_TALL:   i32 = 10;
    pub const LAYOUT_WIDE:   i32 = 11;
    pub const LAYOUT_BSP:    i32 = 12;
    pub const LAYOUT_MONO:   i32 = 13;
    pub const RETILE:        i32 = 20;
    pub const QUIT:          i32 = 99;
}

pub fn register_all(modifier_str: &str) {
    let m = parse_modifier(modifier_str);
    let ms = HOT_KEY_MODIFIERS(m.0 | MOD_SHIFT.0);
    unsafe {
        // Focus: MOD+H (prev) and MOD+L (next)
        reg(id::FOCUS_PREV,   m,  VK_H);
        reg(id::FOCUS_NEXT,   m,  VK_L);
        // Swap: MOD+Shift+H/L
        reg(id::SWAP_PREV,    ms, VK_H);
        reg(id::SWAP_NEXT,    ms, VK_L);
        // Float toggle: MOD+Space
        reg(id::TOGGLE_FLOAT, m,  VK_SPACE);
        // Layouts: MOD+1/2/3/4
        reg(id::LAYOUT_TALL,  m,  VK_1);
        reg(id::LAYOUT_WIDE,  m,  VK_2);
        reg(id::LAYOUT_BSP,   m,  VK_3);
        reg(id::LAYOUT_MONO,  m,  VK_4);
        // Retile: MOD+Enter
        reg(id::RETILE,       m,  VK_RETURN);
        // Quit: MOD+Q
        reg(id::QUIT,         m,  VK_Q);
    }
}

fn parse_modifier(s: &str) -> HOT_KEY_MODIFIERS {
    match s {
        "win"  => MOD_WIN,
        "ctrl" => MOD_CONTROL,
        _      => MOD_ALT,
    }
}

unsafe fn reg(id: i32, modifiers: HOT_KEY_MODIFIERS, vk: u32) {
    let _ = RegisterHotKey(HWND(std::ptr::null_mut()), id, modifiers, vk);
}
