//! Mostly procedural placeholder textures for UI chips until dedicated art lands.
//!
//! Combat theater loads **`assets/ui/dungeon_theater.png`**; title **`assets/ui/campfire_scene.png`**, **`assets/ui/Fireplace.png`**, and **`assets/ui/fire_glow_radial.png`** (soft campfire bloom).
//! Other icons remain procedural (see `assets/ui/README.md`).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
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
    /// Live delve combat strip (`assets/ui/dungeon_theater.png`).
    pub dungeon_theater: Handle<Image>,
    /// Title campfire hub painting (`assets/ui/campfire_scene.png`).
    pub campfire_scene: Handle<Image>,
    /// Title / campfire fireplace still (`assets/ui/Fireplace.png`).
    pub fireplace: Handle<Image>,
    /// Soft radial additive glow for campfire presentation (`assets/ui/fire_glow_radial.png`).
    pub fire_glow_radial: Handle<Image>,
    /// Hero portrait silhouette (procedural until class art lands).
    pub hero_portrait: Handle<Image>,
}

pub fn register_ui_placeholder_images(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
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
        dungeon_theater: asset_server.load("ui/dungeon_theater.png"),
        campfire_scene: asset_server.load("ui/campfire_scene.png"),
        fireplace: asset_server.load("ui/Fireplace.png"),
        fire_glow_radial: asset_server.load("ui/fire_glow_radial.png"),
        hero_portrait: add(gen_hero_portrait()),
    });
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

/// Shoulder-up silhouette for hero identity cards until portrait art lands.
fn gen_hero_portrait() -> Image {
    gen_rgba(48, |x, y| {
        let xf = x as f32 / 48.0;
        let yf = y as f32 / 48.0;
        let head = (xf - 0.5).powi(2) + (yf - 0.28).powi(2) < 0.045;
        let shoulders = yf > 0.52 && yf < 0.88 && xf > 0.12 && xf < 0.88;
        if head || shoulders {
            return [200, 185, 165, 255];
        }
        [0, 0, 0, 0]
    })
}
