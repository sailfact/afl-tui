//! Renders club emblems ([`crate::teams::Emblem`]) into the scoreboard.
//!
//! Each emblem is rasterised to an in-memory [`image::RgbaImage`] and handed to
//! `ratatui-image`, which picks the best terminal graphics protocol (Kitty in
//! Ghostty, Sixel, …) and falls back to Unicode half-blocks elsewhere. Encoded
//! protocols are cached per (club, cell-area) so we only re-encode on resize.

use std::collections::HashMap;

use image::{DynamicImage, Rgba, RgbaImage};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::Image;
use ratatui_image::Resize;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::Protocol;

use crate::teams::{Emblem, Glyph, Rgb, Team};

/// Pixel dimensions of the square canvas every emblem is drawn on.
const SIZE: u32 = 48;

pub struct LogoRenderer {
    /// `None` if the terminal couldn't be queried; emblems are then skipped.
    picker: Option<Picker>,
    cache: HashMap<(&'static str, u16, u16), Protocol>,
}

impl LogoRenderer {
    /// Query the terminal once for its graphics capabilities. Falls back to
    /// half-blocks if the query fails (e.g. a non-interactive terminal).
    pub fn new() -> Self {
        let picker = Picker::from_query_stdio()
            .ok()
            .or_else(|| Some(Picker::halfblocks()));
        Self {
            picker,
            cache: HashMap::new(),
        }
    }

    /// Draw `team`'s emblem to fit `area`. No-op if the area is empty or the
    /// image protocol couldn't be built.
    pub fn draw(&mut self, frame: &mut Frame, team: &Team, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let Some(picker) = self.picker.as_ref() else {
            return;
        };
        let key = (team.nickname, area.width, area.height);
        if let std::collections::hash_map::Entry::Vacant(e) = self.cache.entry(key) {
            let img = DynamicImage::ImageRgba8(rasterize(&team.emblem));
            match picker.new_protocol(img, area, Resize::Fit(None)) {
                Ok(p) => {
                    e.insert(p);
                }
                Err(_) => return,
            }
        }
        if let Some(protocol) = self.cache.get(&key) {
            frame.render_widget(Image::new(protocol), area);
        }
    }
}

impl Default for LogoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Rasterise a club emblem to an RGBA image. Exposed for tooling/tests.
pub fn emblem_image(team: &Team) -> RgbaImage {
    rasterize(&team.emblem)
}

fn px(c: Rgb) -> Rgba<u8> {
    Rgba([c.0, c.1, c.2, 255])
}

/// Paint an emblem onto a fresh `SIZE`×`SIZE` canvas.
fn rasterize(emblem: &Emblem) -> RgbaImage {
    let mut img = RgbaImage::new(SIZE, SIZE);
    match emblem {
        Emblem::VStripes(colors) => stripes(&mut img, colors, true),
        Emblem::HBands(colors) => bands(&mut img, colors),
        Emblem::Sash { bg, stripe } => sash(&mut img, *bg, *stripe),
        Emblem::Vee { bg, fg } => vee(&mut img, *bg, *fg),
        Emblem::Sun { core, ray } => sun(&mut img, *core, *ray),
        Emblem::Anchor { bg, fg } => bitmap(&mut img, *bg, *fg, ANCHOR),
        Emblem::Letter { bg, fg, glyph } => bitmap(&mut img, *bg, *fg, glyph_rows(*glyph)),
    }
    img
}

fn fill(img: &mut RgbaImage, c: Rgb) {
    let p = px(c);
    for pixel in img.pixels_mut() {
        *pixel = p;
    }
}

/// Six guernsey stripes cycling `colors`. `vertical` true → vertical bars.
fn stripes(img: &mut RgbaImage, colors: &[Rgb], vertical: bool) {
    const N: u32 = 6;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let axis = if vertical { x } else { y };
            let idx = (axis * N / SIZE) as usize % colors.len();
            img.put_pixel(x, y, px(colors[idx]));
        }
    }
}

/// One equal-height horizontal band per colour.
fn bands(img: &mut RgbaImage, colors: &[Rgb]) {
    let n = colors.len() as u32;
    for y in 0..SIZE {
        let idx = (y * n / SIZE) as usize;
        let c = px(colors[idx.min(colors.len() - 1)]);
        for x in 0..SIZE {
            img.put_pixel(x, y, c);
        }
    }
}

/// Diagonal sash from top-right to bottom-left over a solid tile.
fn sash(img: &mut RgbaImage, bg: Rgb, stripe: Rgb) {
    fill(img, bg);
    let half = (SIZE as f32) * 0.20;
    for y in 0..SIZE {
        for x in 0..SIZE {
            // x + y is constant along the anti-diagonal; centre it on the tile.
            let d = (x as f32 + y as f32) - (SIZE as f32 - 1.0);
            if d.abs() <= half {
                img.put_pixel(x, y, px(stripe));
            }
        }
    }
}

/// The Swans' white V: two arms meeting low-centre.
fn vee(img: &mut RgbaImage, bg: Rgb, fg: Rgb) {
    fill(img, bg);
    let w = SIZE as f32;
    let apex_y = w * 0.72;
    let slope = apex_y / (w / 2.0);
    let half = w * 0.13;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let xf = x as f32;
            let yf = y as f32;
            // Left arm: y = slope*x ; right arm mirrors. Only above the apex.
            let on_left = (yf - slope * xf).abs() <= half && xf <= w / 2.0;
            let on_right = (yf - slope * (w - 1.0 - xf)).abs() <= half && xf >= w / 2.0;
            if (on_left || on_right) && yf <= apex_y + half {
                img.put_pixel(x, y, px(fg));
            }
        }
    }
}

/// Sunburst: filled disc (ray-colour rim, core-colour centre) plus eight spokes.
fn sun(img: &mut RgbaImage, core: Rgb, ray: Rgb) {
    let cx = (SIZE as f32 - 1.0) / 2.0;
    let cy = cx;
    let r = SIZE as f32 * 0.30;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= r * 0.6 {
                img.put_pixel(x, y, px(core));
            } else if dist <= r {
                img.put_pixel(x, y, px(ray));
            } else if dist >= r * 1.15 && dist <= r * 1.7 {
                // Spokes every 45°: keep pixels close to a spoke angle.
                let ang = dy.atan2(dx);
                let step = std::f32::consts::FRAC_PI_4;
                let nearest = (ang / step).round() * step;
                if (ang - nearest).abs() < 0.18 {
                    img.put_pixel(x, y, px(ray));
                }
            }
        }
    }
}

/// Scale a char bitmap (`' '`/`'.'` = background, anything else = foreground)
/// over a solid background tile, using nearest-neighbour to fill the canvas.
fn bitmap(img: &mut RgbaImage, bg: Rgb, fg: Rgb, rows: &[&str]) {
    fill(img, bg);
    let rh = rows.len() as u32;
    let rw = rows.iter().map(|r| r.len()).max().unwrap_or(0) as u32;
    if rh == 0 || rw == 0 {
        return;
    }
    let fgp = px(fg);
    for y in 0..SIZE {
        let sy = (y * rh / SIZE) as usize;
        let row = rows[sy.min(rows.len() - 1)].as_bytes();
        for x in 0..SIZE {
            let sx = (x * rw / SIZE) as usize;
            let ch = row.get(sx).copied().unwrap_or(b' ');
            if ch != b' ' && ch != b'.' {
                img.put_pixel(x, y, fgp);
            }
        }
    }
}

fn glyph_rows(g: Glyph) -> &'static [&'static str] {
    match g {
        Glyph::C => LETTER_C,
        Glyph::G => LETTER_G,
    }
}

const LETTER_C: &[&str] = &[
    "................",
    "....XXXXXX......",
    "...XXXXXXXX.....",
    "..XXX....XXX....",
    "..XXX.....XX....",
    "..XXX...........",
    "..XXX...........",
    "..XXX...........",
    "..XXX...........",
    "..XXX...........",
    "..XXX.....XX....",
    "..XXX....XXX....",
    "...XXXXXXXX.....",
    "....XXXXXX......",
    "................",
    "................",
];

const LETTER_G: &[&str] = &[
    "................",
    "....XXXXXX......",
    "...XXXXXXXX.....",
    "..XXX....XXX....",
    "..XXX.....XX....",
    "..XXX...........",
    "..XXX...XXXX....",
    "..XXX...XXXX....",
    "..XXX......XX...",
    "..XXX......XX...",
    "..XXX.....XXX...",
    "...XXX...XXX....",
    "...XXXXXXXX.....",
    "....XXXXXX......",
    "................",
    "................",
];

const ANCHOR: &[&str] = &[
    ".......XX.......",
    "......X..X......",
    "......X..X......",
    "......XXXX......",
    ".......XX.......",
    "....XXXXXXXX....",
    ".......XX.......",
    ".......XX.......",
    ".X.....XX.....X.",
    ".XX....XX....XX.",
    "..XX...XX...XX..",
    "...XX..XX..XX...",
    "....XXXXXXXX....",
    "......XXXX......",
    ".......XX.......",
    "................",
];
