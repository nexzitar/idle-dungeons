//! Procedurally generated placeholder textures for UI until real art lands.
//!
//! Replace handles in [`UiPlaceholderImages`] with `AssetServer::load("ui/...png")` once files exist
//! under `assets/` (see `assets/ui/README.md`).

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

const SZ: u32 = 32;

#[derive(Resource, Clone)]
pub struct UiPlaceholderImages {
    pub skill_active: Handle<Image>,
    pub skill_passive: Handle<Image>,
    pub skill_empty: Handle<Image>,
    pub skill_locked: Handle<Image>,
    pub gold_coin: Handle<Image>,
    pub salvage_shard: Handle<Image>,
    pub item_generic: Handle<Image>,
    /// Tiny chip for header stats (skills / depth / speed).
    pub stat_chip: Handle<Image>,
    /// Wide pixel stone strip for combat theater backdrop (`Image` tile).
    pub dungeon_theater: Handle<Image>,
}

pub fn register_ui_placeholder_images(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    let mut add = |img: Image| images.add(img);
    commands.insert_resource(UiPlaceholderImages {
        skill_active: add(gen_skill_active()),
        skill_passive: add(gen_skill_passive()),
        skill_empty: add(gen_skill_empty()),
        skill_locked: add(gen_skill_locked()),
        gold_coin: add(gen_gold_coin()),
        salvage_shard: add(gen_salvage_shard()),
        item_generic: add(gen_item_chest()),
        stat_chip: add(gen_stat_chip()),
        dungeon_theater: add(gen_dungeon_theater()),
    });
}

fn gen_rgba_rect(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let px = f(x, y);
            let i = ((y * w + x) * 4) as usize;
            data[i..i + 4].copy_from_slice(&px);
        }
    }
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Horizontal tile: muted stone blocks with mortar — reads as pixel dungeon wall behind combat.
fn gen_dungeon_theater() -> Image {
    let w = 256u32;
    let h = 96u32;
    gen_rgba_rect(w, h, |x, y| {
        let bx = x / 16;
        let by = y / 12;
        let edge = (x % 16 == 0) || (y % 12 == 0);
        let mut stone = if edge {
            [10u8, 9u8, 12u8, 255u8]
        } else {
            [32u8, 28u8, 34u8, 255u8]
        };
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() / (cx * 1.15);
        let v = (1.0 - d.min(1.0)) * 0.28 + 0.72;
        stone[0] = (stone[0] as f32 * v) as u8;
        stone[1] = (stone[1] as f32 * v) as u8;
        stone[2] = (stone[2] as f32 * v) as u8;
        if !edge && (x ^ y ^ bx ^ by).count_ones() % 6 == 0 {
            stone[0] = stone[0].saturating_add(22);
            stone[1] = stone[1].saturating_add(18);
            stone[2] = stone[2].saturating_add(14);
        }
        stone
    })
}

fn gen_rgba(w: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Image {
    let mut data = vec![0u8; (w * w * 4) as usize];
    for y in 0..w {
        for x in 0..w {
            let px = f(x, y);
            let i = ((y * w + x) * 4) as usize;
            data[i..i + 4].copy_from_slice(&px);
        }
    }
    Image::new(
        Extent3d {
            width: w,
            height: w,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn gen_skill_active() -> Image {
    gen_rgba(SZ, |x, y| {
        let border = x < 2 || y < 2 || x >= SZ - 2 || y >= SZ - 2;
        if border {
            return [200, 165, 90, 255];
        }
        let mut base = [28, 62, 58, 255];
        // subtle diagonal highlight (slash)
        if x + y > 38 && x + y < 46 && x > 4 && y > 4 && x < SZ - 5 && y < SZ - 5 {
            base = [55, 110, 98, 255];
        }
        base
    })
}

fn gen_skill_passive() -> Image {
    gen_rgba(SZ, |x, y| {
        let cx = 16.0_f32;
        let cy = 16.0_f32;
        let d = (x as f32 - cx).abs() + (y as f32 - cy).abs();
        if d < 5.5 {
            return [140, 90, 180, 255];
        }
        if d < 9.0 {
            return [70, 45, 95, 255];
        }
        [35, 26, 45, 255]
    })
}

fn gen_skill_empty() -> Image {
    gen_rgba(SZ, |x, y| {
        let bg = [40, 38, 42, 255];
        if x == y || x + y == SZ - 1 {
            if x > 6 && x < SZ - 7 {
                return [58, 55, 60, 255];
            }
        }
        bg
    })
}

fn gen_skill_locked() -> Image {
    gen_rgba(SZ, |x, y| {
        let frame = x < 3 || y < 3 || x >= SZ - 3 || y >= SZ - 3;
        if frame {
            return [90, 75, 50, 255];
        }
        // lock body
        if x >= 12 && x <= 19 && y >= 12 && y <= 24 {
            return [210, 185, 70, 255];
        }
        if x >= 10 && x <= 21 && y >= 8 && y <= 14 {
            return [160, 150, 140, 255];
        }
        [22, 20, 24, 255]
    })
}

fn gen_gold_coin() -> Image {
    gen_rgba(SZ, |x, y| {
        let cx = 16.0_f32;
        let cy = 16.0_f32;
        let r = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
        if r > 14.0 {
            return [0, 0, 0, 0];
        }
        let t = 1.0 - (r / 14.0).clamp(0.0, 1.0);
        let u = (t * 255.0) as u8;
        [220, 175, 55 + u / 4, 255]
    })
}

fn gen_salvage_shard() -> Image {
    gen_rgba(SZ, |x, y| {
        let cx = 16.0_f32;
        let cy = 16.0_f32;
        let d = (x as f32 - cx).abs() + (y as f32 - cy).abs();
        if d > 13.5 {
            return [0, 0, 0, 0];
        }
        let t = (d / 13.5).clamp(0.0, 1.0);
        let r = (60.0 + (1.0 - t) * 40.0) as u8;
        let g = (160.0 + (1.0 - t) * 80.0) as u8;
        let b = (180.0 + (1.0 - t) * 60.0) as u8;
        [r, g, b, 255]
    })
}

fn gen_item_chest() -> Image {
    gen_rgba(SZ, |x, y| {
        let lid = y >= 8 && y < 14 && x >= 6 && x <= 25;
        let boxy = y >= 14 && y <= 26 && x >= 5 && x <= 26;
        let band = y >= 18 && y <= 20 && x >= 5 && x <= 26;
        if band {
            return [160, 120, 60, 255];
        }
        if lid {
            return [110, 70, 45, 255];
        }
        if boxy {
            return [85, 52, 35, 255];
        }
        [0, 0, 0, 0]
    })
}

fn gen_stat_chip() -> Image {
    gen_rgba(16, |x, y| {
        let cx = 8.0_f32;
        let cy = 8.0_f32;
        let r = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
        if r > 7.0 {
            return [0, 0, 0, 0];
        }
        [120, 112, 105, 255]
    })
}
