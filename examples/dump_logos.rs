//! Renders all 18 club emblems into a single contact-sheet PNG for eyeballing.
//! Run with: cargo run --example dump_logos

use image::{Rgba, RgbaImage};

const TILE: u32 = 48;
const PAD: u32 = 10;
const COLS: u32 = 6;
const BG: [u8; 4] = [11, 11, 20, 255]; // approximate dark terminal background

fn main() {
    let teams = afl_tui::teams::TEAMS;
    let rows = (teams.len() as u32).div_ceil(COLS);
    let cell = TILE + PAD;
    let mut sheet = RgbaImage::from_pixel(COLS * cell + PAD, rows * cell + PAD, Rgba(BG));

    for (i, team) in teams.iter().enumerate() {
        let img = afl_tui::ui::emblem_image(team);
        let gx = (i as u32 % COLS) * cell + PAD;
        let gy = (i as u32 / COLS) * cell + PAD;
        for y in 0..TILE.min(img.height()) {
            for x in 0..TILE.min(img.width()) {
                sheet.put_pixel(gx + x, gy + y, *img.get_pixel(x, y));
            }
        }
    }

    let path = "/tmp/afl-logos.png";
    sheet.save(path).expect("save png");
    println!("wrote {path} ({} teams)", teams.len());
    for (i, t) in teams.iter().enumerate() {
        println!(
            "  cell {} (row {}, col {}): {}",
            i,
            i / COLS as usize,
            i % COLS as usize,
            t.nickname
        );
    }
}
