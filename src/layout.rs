#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self { Self { x, y, w, h } }
}

pub fn compute(layout: &str, screen: Rect, n: usize, gap: i32, main_ratio: f32) -> Vec<Rect> {
    if n == 0 { return vec![]; }
    let g = gap.max(0);
    match layout {
        "monocle" => vec![screen; n],
        "wide"    => wide(screen, n, g, main_ratio),
        "bsp"     => bsp(screen, n, g),
        _         => tall(screen, n, g, main_ratio),
    }
}

fn tall(s: Rect, n: usize, gap: i32, ratio: f32) -> Vec<Rect> {
    if n == 1 { return vec![s]; }
    let main_w = ((s.w as f32) * ratio) as i32;
    let stack_n = (n - 1) as i32;
    // gap solo entre tiles del stack, no en los bordes exteriores
    let tile_h = (s.h - gap * (stack_n - 1)) / stack_n;
    let rem    = s.h - gap * (stack_n - 1) - tile_h * stack_n;

    let mut tiles = vec![Rect::new(s.x, s.y, main_w, s.h)];
    let sx = s.x + main_w + gap;
    let sw = s.w - main_w - gap;
    let mut y = s.y;
    for i in 0..stack_n as usize {
        let h = tile_h + if i == stack_n as usize - 1 { rem } else { 0 };
        tiles.push(Rect::new(sx, y, sw, h));
        y += h + gap;
    }
    tiles
}

fn wide(s: Rect, n: usize, gap: i32, ratio: f32) -> Vec<Rect> {
    if n == 1 { return vec![s]; }
    let main_h = ((s.h as f32) * ratio) as i32;
    let stack_n = (n - 1) as i32;
    let tile_w = (s.w - gap * (stack_n - 1)) / stack_n;
    let rem    = s.w - gap * (stack_n - 1) - tile_w * stack_n;

    let mut tiles = vec![Rect::new(s.x, s.y, s.w, main_h)];
    let sy = s.y + main_h + gap;
    let sh = s.h - main_h - gap;
    let mut x = s.x;
    for i in 0..stack_n as usize {
        let w = tile_w + if i == stack_n as usize - 1 { rem } else { 0 };
        tiles.push(Rect::new(x, sy, w, sh));
        x += w + gap;
    }
    tiles
}

fn bsp(s: Rect, n: usize, gap: i32) -> Vec<Rect> {
    let mut result = vec![s];
    for i in 1..n {
        let last = result[i - 1];
        let (a, b) = if last.w >= last.h {
            let half = (last.w - gap) / 2;
            (Rect::new(last.x, last.y, half, last.h),
             Rect::new(last.x + half + gap, last.y, last.w - half - gap, last.h))
        } else {
            let half = (last.h - gap) / 2;
            (Rect::new(last.x, last.y, last.w, half),
             Rect::new(last.x, last.y + half + gap, last.w, last.h - half - gap))
        };
        result[i - 1] = a;
        result.push(b);
    }
    result
}
