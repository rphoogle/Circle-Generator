//! Circle Generator core: shape layout -> checked brick list -> .brz prefab.
//! Kept separate from any window so other tools can reuse it later.

pub mod geometry;
pub use geometry::{Fill, Kind, Layout, Piece, Shape};

use brdb::{assets, Brick, BrickSize, BrickType as BType, Direction, Rotation, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrickStyle {
    Micro,
    Default,
    Tile,
    Smooth,
}

impl BrickStyle {
    pub fn is_stud(self) -> bool {
        self != BrickStyle::Micro
    }
    /// game units per half-cell
    fn unit(self) -> i32 {
        if self.is_stud() { 5 } else { 1 }
    }
    fn filler(self) -> brdb::BString {
        match self {
            BrickStyle::Micro => assets::bricks::PB_DEFAULT_MICRO_BRICK,
            BrickStyle::Default => assets::bricks::PB_DEFAULT_BRICK,
            BrickStyle::Tile => assets::bricks::PB_DEFAULT_TILE,
            BrickStyle::Smooth => assets::bricks::PB_DEFAULT_SMOOTH_TILE,
        }
    }
    fn wedge(self) -> brdb::BString {
        match self {
            BrickStyle::Micro => assets::bricks::PB_DEFAULT_MICRO_WEDGE,
            BrickStyle::Default => assets::bricks::PB_DEFAULT_SIDE_WEDGE,
            // Tile and Smooth both use the flat-topped side wedge (agreed)
            BrickStyle::Tile | BrickStyle::Smooth => brdb::BString::str("PB_DefaultSideWedgeTile"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub layout: Layout,
    pub style: BrickStyle,
    /// Height: micro bricks (Micro) or plates (stud styles)
    pub height: i32,
    pub color: (u8, u8, u8),
    pub material: String,
    pub name: String,
}

/// The game's limit for one brick: 4000 units from centre to edge
/// (confirmed in game, Nov 2026: 800 studs / 4000 micro bricks long).
const MAX_HALF: i32 = 4000;

pub struct Built {
    pub pieces: Vec<Piece>,
    pub bricks: Vec<Brick>,
    pub wedges: usize,
}

/// Check the settings make sense, in plain words.
pub fn validate(s: &Settings) -> Result<(), String> {
    let l = &s.layout;
    let unit = if s.style.is_stud() { "studs" } else { "micro bricks" };
    let one = if s.style.is_stud() { "stud" } else { "micro brick" };
    if l.width < 2 || l.length < 2 {
        return Err(format!("The size must be at least 2 {unit}."));
    }
    let even = |v: i32| v % 2 == 0;
    match l.shape {
        Shape::Full => {}
        Shape::Half => {
            if !even(l.length) {
                return Err(format!(
                    "Half shapes need an even {}, because the cut runs through the centre and an odd size \
                     would mean cutting a brick in half. Try {} or {} {unit}.",
                    if l.width == l.length { "diameter" } else { "length" },
                    l.length - 1,
                    l.length + 1
                ));
            }
        }
        Shape::Quarter => {
            if !even(l.length) || !even(l.width) {
                return Err(format!(
                    "Quarter shapes need even sizes, because the cuts run through the centre and an odd size \
                     would mean cutting a brick in half."
                ));
            }
        }
    }
    if l.kind == Kind::Ring {
        if l.hole_w < 2 || l.hole_l < 2 {
            return Err(format!("The hole must be at least 2 {unit} across."));
        }
        if l.hole_w >= l.width || l.hole_l >= l.length {
            return Err("The band must be at least 1 thick.".into());
        }
        if (l.width - l.hole_w) % 2 != 0 || (l.length - l.hole_l) % 2 != 0 {
            return Err("internal: the band must be the same thickness on both sides.".into());
        }
    }
    if l.inverse && l.kind != Kind::Hollow && l.wall < 1 {
        return Err(format!("The wall must be at least 1 {one}."));
    }
    if s.height < 1 {
        return Err("The height must be at least 1.".into());
    }
    let half_z = if s.style.is_stud() { 2 * s.height } else { s.height };
    if half_z > MAX_HALF {
        return Err("That is taller than the game allows for one brick (4000 micro bricks / 2000 plates).".into());
    }
    Ok(())
}

pub fn build(s: &Settings) -> Result<Built, String> {
    validate(s)?;
    let pieces = geometry::layout(&s.layout)?;
    let u = s.style.unit();
    let half_z = if s.style.is_stud() { 2 * s.height } else { s.height };
    let max_len = (2 * MAX_HALF / u) / 2 * 2; // longest brick in half-cells, kept even
    let mut bricks = vec![];
    let mut wedges = 0;
    for p in &pieces {
        let (x0, y0, x1, y1) = p.bbox();
        if (x1 - x0) % 2 != 0 || (y1 - y0) % 2 != 0 || (x0 + x1) % 2 != 0 || (y0 + y1) % 2 != 0 {
            return Err(format!("internal: piece {p:?} is off the grid"));
        }
        let mk = |asset: brdb::BString, a0: i32, a1: i32, b0: i32, b1: i32, rot: u8| {
            let (hx, hy) = ((a1 - a0) * u / 2, (b1 - b0) * u / 2);
            let (sx, sy) = if rot % 2 == 1 { (hy, hx) } else { (hx, hy) };
            Brick {
                asset: BType::Procedural { asset, size: BrickSize { x: sx as u16, y: sy as u16, z: half_z as u16 } },
                position: ((a0 + a1) * u / 2, (b0 + b1) * u / 2, 0).into(),
                rotation: [Rotation::Deg0, Rotation::Deg90, Rotation::Deg180, Rotation::Deg270][rot as usize],
                direction: Direction::ZPositive,
                color: s.color.into(),
                material: brdb::BString::from(s.material.clone()),
                material_intensity: 5,
                ..Default::default()
            }
        };
        match *p {
            Piece::Tri { corner, .. } => {
                if (x1 - x0) > max_len || (y1 - y0) > max_len {
                    return Err("This circle is too big: one of its edge wedges would be longer than the game allows.".into());
                }
                bricks.push(mk(s.style.wedge(), x0, x1, y0, y1, corner));
                wedges += 1;
            }
            Piece::Rect { .. } => {
                // split anything longer than the game allows into equal pieces
                let cuts = |lo: i32, hi: i32| -> Vec<(i32, i32)> {
                    let len = hi - lo; // half-cells, even
                    let n = (len + max_len - 1) / max_len; // number of pieces
                    let cells = len / 2;
                    let mut v = vec![];
                    let mut a = lo;
                    for i in 0..n {
                        let c = cells / n + if i < cells % n { 1 } else { 0 };
                        v.push((a, a + 2 * c));
                        a += 2 * c;
                    }
                    v
                };
                for (a, a2) in cuts(x0, x1) {
                    for (b, b2) in cuts(y0, y1) {
                        bricks.push(mk(s.style.filler(), a, a2, b, b2, 0));
                    }
                }
            }
        }
    }
    Ok(Built { pieces, bricks, wedges })
}

/// Build a prefab world, centred like the game's own copies.
pub fn to_world(s: &Settings, built: &Built) -> World {
    let mut w = World::new();
    w.bricks = built.bricks.clone();
    if let Some((min, max)) = w.brick_bounds() {
        let (cx, cy, cz) = ((min.x + max.x) / 2, (min.y + max.y) / 2, (min.z + max.z) / 2);
        for b in &mut w.bricks {
            b.position.x -= cx;
            b.position.y -= cy;
            b.position.z -= cz;
        }
    }
    w.meta.bundle.name = s.name.clone();
    w.meta.bundle.description = "Made with Circle Generator".into();
    w.meta.bundle.game_version = "CL16507".into();
    w.make_prefab();
    w
}

pub fn write_brz(s: &Settings, path: &std::path::Path) -> Result<Built, String> {
    let built = build(s)?;
    let w = to_world(s, &built);
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    w.write_brz(path).map_err(|e| e.to_string())?;
    Ok(built)
}

pub fn to_brz_bytes(s: &Settings) -> Result<(Built, Vec<u8>), String> {
    let built = build(s)?;
    let w = to_world(s, &built);
    let bytes = w.to_brz_vec().map_err(|e| e.to_string())?;
    Ok((built, bytes))
}

pub mod check;
