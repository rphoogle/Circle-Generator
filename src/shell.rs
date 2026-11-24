//! Shell tab: curved shells (cones, dishes, spheres, balloons) built from flat
//! tapered panels, each panel on its own entity.
//!
//! In plain English: the shape is a curve (its side outline) spun around the
//! vertical axis. The curve is cut into rings and the circle into strips, so
//! every panel is a flat trapezoid (wider at one end). Each panel is one
//! rectangle of micro bricks plus a micro wedge down each slanted side, built
//! on its own entity and placed at its exact 3D position and tilt.
//!
//! With bearings on, the panels are held the same way as the user's own dome:
//! a column of bearings at the centre turns each strip to face its direction,
//! and every panel carries a bearing on its outer end that holds the next
//! panel in the strip at the right tilt. The bearing joints are worked out so
//! the game puts every panel exactly where the maths says (rule measured from
//! the user's Reflective Dome and balloon prefabs: 288 joints, error < 0.001).
//!
//! Units: game units. 1 micro brick = 2 units, 1 stud = 10 units.

use brdb::{
    assets::LiteralComponent, AsBrdbValue, Brick, BrickSize, BrickType, Direction, Entity, Position, Quat4f,
    Rotation, Vector3f, World,
};

pub const BEARING: &str = "B_Joint_RigidBearing_Micro";
pub const MICRO_BRICK: &str = "PB_DefaultMicroBrick";
pub const MICRO_WEDGE: &str = "PB_DefaultMicroWedge";
pub const SIDE_WEDGE: &str = "PB_DefaultSideWedge";
const MAX_HALF: i32 = 4000;

// ------------------------------------------------------------------ settings

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    Cone,
    Dish,
    Sphere,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SphereInput {
    /// sphere radius + start and end angle (0 = the pole where the strips meet)
    Angles,
    /// rim radius + depth (a cap, up to almost a full sphere)
    RadiusDepth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spacing {
    EqualLength,
    EqualHeight,
    EqualAngle,
}

/// Which face is kept smooth where panels meet at an angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seam {
    /// outside (away from the axis) faces meet exactly; overlaps hidden inside
    Outside,
    /// inside faces meet exactly; small gaps on the outside
    Inside,
    /// split evenly: tiny gaps outside, tiny overlaps inside
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColourMode {
    One,
    PerRing,
    PerStrip,
    Checker,
}

/// Where a bearing goes on 1-micro-thick panels (it can't hide inside).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BearingSide {
    Inside,
    Outside,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShellSettings {
    pub curve: Curve,
    // cone (micro bricks)
    pub cone_bottom: i32,
    pub cone_top: i32,
    pub cone_height: i32,
    // dish
    pub dish_radius: i32,
    pub dish_depth: i32,
    // sphere
    pub sphere_input: SphereInput,
    pub sphere_radius: i32,
    pub start_deg: i32,
    pub end_deg: i32,
    pub cap_radius: i32,
    pub cap_depth: i32,
    /// custom outline points (radius, height) in micro bricks, from the end
    /// where the strips meet
    pub points: Vec<(f64, f64)>,
    pub upside_down: bool,
    pub strips: u32,
    pub rings: u32,
    pub spacing: Spacing,
    /// micro bricks
    pub thickness: i32,
    pub seam: Seam,
    pub bearings: bool,
    pub hide_bearings: bool,
    pub bearing_side: BearingSide,
    pub column_inside: bool,
    pub colour_mode: ColourMode,
    pub colours: Vec<[u8; 3]>,
    pub material: String,
    pub intensity: u8,
}

impl Default for ShellSettings {
    fn default() -> Self {
        ShellSettings {
            curve: Curve::Dish,
            cone_bottom: 40,
            cone_top: 120,
            cone_height: 60,
            dish_radius: 320,
            dish_depth: 105,
            sphere_input: SphereInput::Angles,
            sphere_radius: 150,
            start_deg: 0,
            end_deg: 180,
            cap_radius: 150,
            cap_depth: 60,
            points: balloon_points(),
            upside_down: false,
            strips: 24,
            rings: 8,
            spacing: Spacing::EqualLength,
            thickness: 3,
            seam: Seam::Outside,
            bearings: true,
            hide_bearings: true,
            bearing_side: BearingSide::Inside,
            column_inside: false,
            colour_mode: ColourMode::One,
            colours: vec![[255, 255, 255], [224, 85, 75]],
            material: "BMC_Plastic".into(),
            intensity: 5,
        }
    }
}

/// The outline of the user's hot air balloon (measured from
/// Balloon_No_Gravity.brz, one gore), in micro bricks.
pub fn balloon_points() -> Vec<(f64, f64)> {
    vec![
        (20.0, 0.0),
        (33.0, 39.0),
        (52.0, 77.0),
        (76.0, 111.0),
        (93.5, 132.0),
        (98.0, 158.0),
        (97.0, 183.0),
        (87.0, 207.0),
        (71.0, 228.0),
        (52.5, 248.0),
        (30.0, 254.0),
    ]
}

// ------------------------------------------------------------------ maths

pub type V = [f64; 3];
/// 3x3 matrix, rows; columns are the local axes in world space.
pub type M = [[f64; 3]; 3];

fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: V, s: f64) -> V {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V, b: V) -> V {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn norm(a: V) -> f64 {
    dot(a, a).sqrt()
}
pub fn mv(m: &M, v: V) -> V {
    [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
}
pub fn mm(a: &M, b: &M) -> M {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    r
}
pub fn tr(a: &M) -> M {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[j][i];
        }
    }
    r
}
fn from_cols(x: V, y: V, z: V) -> M {
    [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]]
}
pub fn rz(deg: f64) -> M {
    let (s, c) = deg.to_radians().sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}
const IDENT: M = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
/// Hinge bearing orientation (Y+, rotation 0): local x -> +Z, y -> +X, z (axis) -> +Y.
/// Measured from the user's dome.
pub const D_HINGE: M = [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]];
/// Bearing orientation X+, rotation 0 (byte 0): x -> +Z, y -> -Y, z (axis) -> +X. Measured from the balloon.
pub const D_XPOS: M = [[0.0, 0.0, 1.0], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0]];
/// The rotation for a bearing orientation byte (only the measured ones).
pub fn d_of(ori: u8) -> M {
    match ori {
        8 => D_HINGE,
        0 => D_XPOS,
        _ => IDENT,
    }
}

pub fn quat_from(m: &M) -> [f64; 4] {
    // returns x, y, z, w
    let t = m[0][0] + m[1][1] + m[2][2];
    let (x, y, z, w);
    if t > 0.0 {
        let s = (t + 1.0).sqrt() * 2.0;
        w = 0.25 * s;
        x = (m[2][1] - m[1][2]) / s;
        y = (m[0][2] - m[2][0]) / s;
        z = (m[1][0] - m[0][1]) / s;
    } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 2.0;
        w = (m[2][1] - m[1][2]) / s;
        x = 0.25 * s;
        y = (m[0][1] + m[1][0]) / s;
        z = (m[0][2] + m[2][0]) / s;
    } else if m[1][1] > m[2][2] {
        let s = (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt() * 2.0;
        w = (m[0][2] - m[2][0]) / s;
        x = (m[0][1] + m[1][0]) / s;
        y = 0.25 * s;
        z = (m[1][2] + m[2][1]) / s;
    } else {
        let s = (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt() * 2.0;
        w = (m[1][0] - m[0][1]) / s;
        x = (m[0][2] + m[2][0]) / s;
        y = (m[1][2] + m[2][1]) / s;
        z = 0.25 * s;
    }
    let n = (x * x + y * y + z * z + w * w).sqrt();
    [x / n, y / n, z / n, w / n]
}

pub fn mat_from_quat(q: [f64; 4]) -> M {
    let [x, y, z, w] = q;
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w)],
        [2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w)],
        [2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

/// Where the game puts an entity held by a Rigid Bearing (measured rule).
/// `parent` = pose of the grid the bearing is on; `pos`/`d` = the bearing's
/// position and orientation in that grid; `angle` = its target angle.
pub fn joint_child(parent: (V, M), pos: V, d: &M, angle: f64, off: V, q: &M) -> (V, M) {
    let f = mm(d, &rz(-angle));
    let origin = sub(pos, mv(d, [0.0, 0.0, 1.0])); // the bearing's mounting face
    let loc = add(parent.0, mv(&parent.1, add(origin, mv(&f, off))));
    let rot = mm(&mm(&parent.1, &f), q);
    (loc, rot)
}

/// The joint offset / rotation that puts the child exactly at `child`.
pub fn joint_for(parent: (V, M), pos: V, d: &M, angle: f64, child: (V, M)) -> (V, M) {
    let f = mm(d, &rz(-angle));
    let ft = tr(&f);
    let q = mm(&mm(&ft, &tr(&parent.1)), &child.1);
    let origin = sub(pos, mv(d, [0.0, 0.0, 1.0]));
    let local = mv(&tr(&parent.1), sub(child.0, parent.0));
    let off = mv(&ft, sub(local, origin));
    (off, q)
}

fn wrap_deg(a: f64) -> f64 {
    let mut a = a % 360.0;
    if a <= -180.0 {
        a += 360.0;
    }
    if a > 180.0 {
        a -= 360.0;
    }
    a
}

// ------------------------------------------------------------------ outline

/// Dense outline (radius, height) in game units, from the strips' meeting end.
pub fn outline(s: &ShellSettings) -> Result<Vec<(f64, f64)>, String> {
    const N: usize = 3000;
    let mut p: Vec<(f64, f64)> = vec![];
    match s.curve {
        Curve::Cone => {
            if s.cone_bottom < 0 || s.cone_top < 0 {
                return Err("Cone sizes can't be negative.".into());
            }
            if s.cone_height < 1 {
                return Err("The cone needs a height of at least 1 micro brick.".into());
            }
            if s.cone_bottom == s.cone_top && s.cone_height == 0 {
                return Err("That cone has no size.".into());
            }
            let (r0, r1, h) = (s.cone_bottom as f64 * 2.0, s.cone_top as f64 * 2.0, s.cone_height as f64 * 2.0);
            for i in 0..=N {
                let t = i as f64 / N as f64;
                p.push((r0 + (r1 - r0) * t, h * t));
            }
        }
        Curve::Dish => {
            if s.dish_radius < 2 {
                return Err("The dish radius must be at least 2 micro bricks.".into());
            }
            if s.dish_depth < 1 {
                return Err("The dish depth must be at least 1 micro brick.".into());
            }
            let (r, d) = (s.dish_radius as f64 * 2.0, s.dish_depth as f64 * 2.0);
            for i in 0..=N {
                let t = i as f64 / N as f64;
                p.push((r * t, d * t * t));
            }
        }
        Curve::Sphere => {
            let (rho, a0, a1) = match s.sphere_input {
                SphereInput::Angles => {
                    if s.sphere_radius < 2 {
                        return Err("The sphere radius must be at least 2 micro bricks.".into());
                    }
                    if !(0..180).contains(&s.start_deg) || !(1..=180).contains(&s.end_deg) || s.start_deg >= s.end_deg {
                        return Err("Angles go from 0 (the pole where the strips meet) to 180 (the far pole), and the start must be smaller than the end.".into());
                    }
                    (s.sphere_radius as f64 * 2.0, (s.start_deg as f64).to_radians(), (s.end_deg as f64).to_radians())
                }
                SphereInput::RadiusDepth => {
                    if s.cap_radius < 2 || s.cap_depth < 1 {
                        return Err("The rim radius must be at least 2 and the depth at least 1 micro brick.".into());
                    }
                    let (r, d) = (s.cap_radius as f64 * 2.0, s.cap_depth as f64 * 2.0);
                    let rho = (r * r + d * d) / (2.0 * d);
                    let end = ((rho - d) / rho).clamp(-1.0, 1.0).acos();
                    (rho, 0.0, end)
                }
            };
            for i in 0..=N {
                let a = a0 + (a1 - a0) * i as f64 / N as f64;
                p.push((rho * a.sin(), rho - rho * a.cos()));
            }
        }
        Curve::Custom => {
            let pts: Vec<(f64, f64)> = s.points.iter().map(|&(r, z)| (r * 2.0, z * 2.0)).collect();
            if pts.len() < 2 {
                return Err("The custom outline needs at least 2 points.".into());
            }
            if pts.iter().any(|&(r, _)| r < 0.0) {
                return Err("Custom points can't have a negative radius.".into());
            }
            for w in pts.windows(2) {
                if ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt() < 1.0 {
                    return Err("Two custom points are at the same place.".into());
                }
            }
            // centripetal Catmull-Rom through the points
            let n = pts.len();
            let per = (N / (n - 1)).max(20);
            let get = |i: isize| -> (f64, f64) {
                if i < 0 {
                    let (a, b) = (pts[0], pts[1]);
                    (2.0 * a.0 - b.0, 2.0 * a.1 - b.1)
                } else if i as usize >= n {
                    let (a, b) = (pts[n - 1], pts[n - 2]);
                    (2.0 * a.0 - b.0, 2.0 * a.1 - b.1)
                } else {
                    pts[i as usize]
                }
            };
            for i in 0..n - 1 {
                let (p0, p1, p2, p3) = (get(i as isize - 1), get(i as isize), get(i as isize + 1), get(i as isize + 2));
                let dt = |a: (f64, f64), b: (f64, f64)| (((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()).sqrt().max(1e-6);
                let t0 = 0.0;
                let t1 = t0 + dt(p0, p1);
                let t2 = t1 + dt(p1, p2);
                let t3 = t2 + dt(p2, p3);
                for k in 0..per {
                    let t = t1 + (t2 - t1) * k as f64 / per as f64;
                    let l = |a: (f64, f64), b: (f64, f64), ta: f64, tb: f64| {
                        let u = (t - ta) / (tb - ta);
                        (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u)
                    };
                    let a1 = l(p0, p1, t0, t1);
                    let a2 = l(p1, p2, t1, t2);
                    let a3 = l(p2, p3, t2, t3);
                    let b1 = l(a1, a2, t0, t2);
                    let b2 = l(a2, a3, t1, t3);
                    let c = l(b1, b2, t1, t2);
                    p.push((c.0.max(0.0), c.1));
                }
            }
            p.push(pts[n - 1]);
        }
    }
    if s.upside_down {
        for q in &mut p {
            q.1 = -q.1;
        }
    }
    Ok(p)
}

/// Cut the outline into `rings` pieces. Returns the ring edge points and an
/// optional note.
pub fn ring_points(s: &ShellSettings, line: &[(f64, f64)]) -> (Vec<(f64, f64)>, Option<String>) {
    let n = line.len();
    let seg_len: Vec<f64> = line.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).collect();
    let mut note = None;
    let mut metric: Vec<f64> = match s.spacing {
        Spacing::EqualLength => seg_len.clone(),
        Spacing::EqualHeight => line.windows(2).map(|w| (w[1].1 - w[0].1).abs()).collect(),
        Spacing::EqualAngle => {
            let ang: Vec<f64> = line.windows(2).map(|w| (w[1].1 - w[0].1).atan2(w[1].0 - w[0].0)).collect();
            let mut m = vec![0.0; n - 1];
            for i in 1..ang.len() {
                let mut d = ang[i] - ang[i - 1];
                while d > std::f64::consts::PI {
                    d -= 2.0 * std::f64::consts::PI;
                }
                while d < -std::f64::consts::PI {
                    d += 2.0 * std::f64::consts::PI;
                }
                m[i] = d.abs();
            }
            m
        }
    };
    let total: f64 = metric.iter().sum();
    let len_total: f64 = seg_len.iter().sum();
    let too_small = match s.spacing {
        Spacing::EqualLength => false,
        Spacing::EqualHeight => total < 1e-6 * len_total.max(1.0) || total < 0.5,
        Spacing::EqualAngle => total < 0.01,
    };
    if too_small {
        note = Some(match s.spacing {
            Spacing::EqualHeight => "This shape has no height, so rings use equal panel length instead.".into(),
            _ => "This shape doesn't curve, so rings use equal panel length instead.".into(),
        });
        metric = seg_len.clone();
    } else if s.spacing != Spacing::EqualLength {
        // mix in a little length so flat or straight stretches still get split
        let tm: f64 = metric.iter().sum();
        for (m, l) in metric.iter_mut().zip(&seg_len) {
            *m += 1e-4 * tm * l / len_total;
        }
    }
    let total: f64 = metric.iter().sum();
    let mut out = vec![line[0]];
    let mut acc = 0.0;
    let mut i = 0;
    for k in 1..s.rings {
        let target = total * k as f64 / s.rings as f64;
        while i < metric.len() && acc + metric[i] < target {
            acc += metric[i];
            i += 1;
        }
        let i2 = i.min(metric.len() - 1);
        let u = if metric[i2] > 0.0 { ((target - acc) / metric[i2]).clamp(0.0, 1.0) } else { 0.0 };
        let (a, b) = (line[i2], line[i2 + 1]);
        out.push((a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u));
    }
    out.push(line[n - 1]);
    (out, note)
}

// ------------------------------------------------------------------ build

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Micro,
    Wedge,
    SideWedge,
    Bearing,
}

/// One brick on a grid: half-size (units), centre (units), orientation byte
/// (direction * 4 + rotation; 16 = upright).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pb {
    pub kind: Kind,
    pub size: (i32, i32, i32),
    pub pos: (i32, i32, i32),
    pub ori: u8,
}

impl Pb {
    /// half-extents along the grid's own axes
    pub fn half(&self) -> (i32, i32, i32) {
        match self.kind {
            Kind::Bearing => (1, 1, 1),
            _ => {
                if self.ori & 1 == 1 {
                    (self.size.1, self.size.0, self.size.2)
                } else {
                    self.size
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Holder {
    /// centre column bearing number k (main grid)
    Column(usize),
    /// the hinge bearing on panel number i
    Panel(usize),
    /// bearing number k (in brick order) on panel number i
    PanelK(usize, usize),
}

#[derive(Clone, Debug)]
pub struct Joint {
    pub holder: Holder,
    pub angle: f64,
    pub off: V,
    pub q: M,
}

#[derive(Clone, Debug)]
pub struct Panel {
    pub ring: usize,
    pub strip: usize,
    /// world pose of the panel's entity (game units, before centring)
    pub loc: V,
    pub rot: M,
    pub bricks: Vec<Pb>,
    pub colour: usize,
    /// the true trapezoid: length, inner width, outer width (units)
    pub length: f64,
    pub w_in: f64,
    pub w_out: f64,
    /// how it is held (None when bearings are off)
    pub joint: Option<Joint>,
}

#[derive(Clone, Debug)]
pub struct ShellResult {
    pub panels: Vec<Panel>,
    /// main grid: column bearings (in strip order) and the base plate
    pub main: Vec<Pb>,
    pub column: Vec<usize>,
    pub outline: Vec<(f64, f64)>,
    pub rings_at: Vec<(f64, f64)>,
    pub warnings: Vec<String>,
    /// side the surface's "inside" is on, in each panel's local z (+1 / -1)
    pub inside_sign: f64,
}

impl ShellResult {
    pub fn brick_count(&self) -> usize {
        self.main.len() + self.panels.iter().map(|p| p.bricks.len()).sum::<usize>()
    }
    pub fn bearing_count(&self) -> usize {
        self.main.iter().filter(|b| b.kind == Kind::Bearing).count()
            + self.panels.iter().flat_map(|p| &p.bricks).filter(|b| b.kind == Kind::Bearing).count()
    }
}

pub fn validate(s: &ShellSettings) -> Result<(), String> {
    if s.strips < 3 {
        return Err("Use at least 3 strips.".into());
    }
    if s.rings < 1 {
        return Err("Use at least 1 ring.".into());
    }
    if s.strips > 720 || s.rings > 400 {
        return Err("That's more panels than this tool allows (720 strips, 400 rings).".into());
    }
    if s.thickness < 1 || s.thickness > 200 {
        return Err("Thickness must be between 1 and 200 micro bricks.".into());
    }
    if s.colours.is_empty() {
        return Err("Pick at least one colour.".into());
    }
    Ok(())
}

fn colour_index(s: &ShellSettings, ring: usize, strip: usize) -> usize {
    let n = s.colours.len().max(1);
    match s.colour_mode {
        ColourMode::One => 0,
        ColourMode::PerRing => ring % n,
        ColourMode::PerStrip => strip % n,
        ColourMode::Checker => (ring + strip) % n,
    }
}

/// Add a box brick covering x0..x1, y0..y1, z0..z1 (units, even sizes).
fn boxb(v: &mut Vec<Pb>, x0: i32, x1: i32, y0: i32, y1: i32, z0: i32, z1: i32) {
    if x1 <= x0 || y1 <= y0 || z1 <= z0 {
        return;
    }
    v.push(Pb {
        kind: Kind::Micro,
        size: ((x1 - x0) / 2, (y1 - y0) / 2, (z1 - z0) / 2),
        pos: ((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2),
        ori: 16,
    });
}

pub fn build_shell(s: &ShellSettings) -> Result<ShellResult, String> {
    validate(s)?;
    let line = outline(s)?;
    let (rp, note) = ring_points(s, &line);
    let mut warnings: Vec<String> = note.into_iter().collect();
    let nr = rp.len() - 1;
    let ns = s.strips as usize;
    let dphi = 360.0 / ns as f64;
    let beta = (dphi / 2.0).to_radians();
    let t = s.thickness;

    // panel poses
    struct Raw {
        loc: V,
        rot: M,
        len: f64,
        w_in: f64,
        w_out: f64,
    }
    let mut raws: Vec<Raw> = vec![];
    for i in 0..nr {
        let (r0, z0) = rp[i];
        let (r1, z1) = rp[i + 1];
        let (m0, m1) = ((r0 * beta.cos(), z0), (r1 * beta.cos(), z1));
        let len = ((m1.0 - m0.0).powi(2) + (m1.1 - m0.1).powi(2)).sqrt();
        if len < 2.0 {
            return Err(format!(
                "Ring {} would be shorter than 1 micro brick. Use fewer rings or a bigger shape.",
                i + 1
            ));
        }
        for j in 0..ns {
            let phi = (j as f64 * dphi).to_radians();
            let (sp, cp) = phi.sin_cos();
            let x = [(m1.0 - m0.0) / len * cp, (m1.0 - m0.0) / len * sp, (m1.1 - m0.1) / len];
            let y = [-sp, cp, 0.0];
            let z = cross(x, y);
            let mid = ((m0.0 + m1.0) / 2.0, (m0.1 + m1.1) / 2.0);
            raws.push(Raw {
                loc: [mid.0 * cp, mid.0 * sp, mid.1],
                rot: from_cols(x, y, z),
                len,
                w_in: 2.0 * r0 * beta.sin(),
                w_out: 2.0 * r1 * beta.sin(),
            });
        }
    }
    // which local z side faces the axis ("inside")
    let mut lean = 0.0;
    for r in &raws {
        let radial = [r.loc[0], r.loc[1], 0.0];
        let rn = norm(radial);
        if rn > 1e-9 {
            let zc = [r.rot[0][2], r.rot[1][2], r.rot[2][2]];
            lean += -dot(zc, scale(radial, 1.0 / rn)) * r.len * (r.w_in + r.w_out);
        }
    }
    let inside_sign = if lean < 0.0 { -1.0 } else { 1.0 };
    let sig = inside_sign as i32;
    // brick layer: z range of the panel in local units
    let zc = match s.seam {
        Seam::Outside => sig * t,
        Seam::Inside => -sig * t,
        Seam::Middle => 0,
    };
    let (zlo, zhi) = (zc - t, zc + t);
    if s.bearings && t == 1 {
        warnings.push(format!(
            "1-micro panels can't hide their bearings: they sit on the {} face{}.",
            if s.bearing_side == BearingSide::Inside { "inside" } else { "outside" },
            if s.hide_bearings { " (hidden, so you won't see them)" } else { "" }
        ));
    }

    let mut panels: Vec<Panel> = vec![];
    let mut max_err: f64 = 0.0;
    for i in 0..nr {
        for j in 0..ns {
            let r = &raws[i * ns + j];
            let has_child = s.bearings && i + 1 < nr;
            let mut v: Vec<Pb> = vec![];
            let lm = ((r.len / 2.0) - 1e-6).ceil().max(1.0) as i32; // half-length (units) = length in micros
            let narrow = r.w_in.min(r.w_out);
            let wide = r.w_in.max(r.w_out);
            let mut a = ((narrow / 2.0) - 1e-6).ceil().max(0.0) as i32; // half-width of the rectangle
            if has_child && a == 0 {
                a = 1;
            }
            let b = (((wide / 2.0 - a as f64) / 2.0) - 1e-6).ceil().max(0.0) as i32; // wedge run (units)
            if a == 0 && b == 0 {
                return Err("A panel would be narrower than 1 micro brick. Use fewer strips or a bigger shape.".into());
            }
            if lm > MAX_HALF || a > MAX_HALF || b > MAX_HALF {
                return Err("A panel would be bigger than the game allows for one brick. Use more strips or rings.".into());
            }
            let outer_wide = r.w_out >= r.w_in;
            // rectangle (shortened at the outer end when it holds a bearing)
            let hole = has_child && t >= 2 && a > 0;
            let x_end = if hole { lm - 2 } else { lm };
            if a > 0 {
                boxb(&mut v, -lm, x_end, -a, a, zlo, zhi);
            }
            let mut bearing_at = None;
            if has_child {
                let yc = if a % 2 == 0 { 1 } else { 0 };
                if hole {
                    let m = (t - 1) / 2;
                    let zb = zlo + 1 + 2 * m;
                    boxb(&mut v, lm - 2, lm, -a, yc - 1, zlo, zhi);
                    boxb(&mut v, lm - 2, lm, yc + 1, a, zlo, zhi);
                    boxb(&mut v, lm - 2, lm, yc - 1, yc + 1, zlo, zb - 1);
                    boxb(&mut v, lm - 2, lm, yc - 1, yc + 1, zb + 1, zhi);
                    bearing_at = Some((lm - 1, yc, zb));
                } else {
                    // 1 micro thick: on a face
                    let side = if s.bearing_side == BearingSide::Inside { sig } else { -sig };
                    let zb = if side > 0 { zhi + 1 } else { zlo - 1 };
                    bearing_at = Some((lm - 1, yc, zb));
                }
            }
            if b > 0 {
                let (cp, cm) = if outer_wide { (1u8, 2u8) } else { (0u8, 3u8) };
                for (yc, corner) in [(a + b, cp), (-(a + b), cm)] {
                    let size = if corner % 2 == 1 { (b, lm, t) } else { (lm, b, t) };
                    v.push(Pb { kind: Kind::Wedge, size, pos: (0, yc, zc), ori: 16 + corner });
                }
            }
            if let Some(p) = bearing_at {
                v.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: p, ori: 8 });
            }
            // how far the bricks stick out past the true trapezoid (never short)
            let e = [
                lm as f64 - r.len / 2.0,
                a as f64 - narrow / 2.0,
                (a + 2 * b) as f64 - wide / 2.0,
            ];
            for x in e {
                if x < -1e-6 {
                    return Err("internal: a panel came out smaller than its shape".into());
                }
                max_err = max_err.max(x);
            }
            panels.push(Panel {
                ring: i,
                strip: j,
                loc: r.loc,
                rot: r.rot,
                bricks: v,
                colour: colour_index(s, i, j),
                length: r.len,
                w_in: r.w_in,
                w_out: r.w_out,
                joint: None,
            });
        }
    }

    // centre column + joints
    let mut main: Vec<Pb> = vec![];
    let mut column: Vec<usize> = vec![];
    if s.bearings {
        let start = rp[0];
        let next_z = rp[1].1;
        let shape_above = next_z >= start.1;
        let up = if s.column_inside { shape_above } else { !shape_above };
        let gap = 2 * t + 2;
        let z0 = if up { start.1.ceil() as i32 + gap } else { start.1.floor() as i32 - gap };
        // bearings stacked from the base plate upwards
        let n = ns as i32;
        let lowest = if up { z0 + 1 } else { z0 - 1 - 2 * (n - 1) };
        main.push(Pb { kind: Kind::Micro, size: (3, 3, 1), pos: (0, 0, lowest - 2), ori: 16 });
        for k in 0..ns {
            // strip k's bearing: nearest the shape first
            let zk = if up { lowest + 2 * k as i32 } else { lowest + 2 * (n - 1 - k as i32) };
            column.push(main.len());
            main.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: (0, 0, zk), ori: 16 });
        }
        for idx in 0..panels.len() {
            let (ring, strip) = (panels[idx].ring, panels[idx].strip);
            let child = (panels[idx].loc, panels[idx].rot);
            let joint = if ring == 0 {
                let pb = main[column[strip]];
                let pos = [pb.pos.0 as f64, pb.pos.1 as f64, pb.pos.2 as f64];
                let angle = wrap_deg(-(strip as f64 * dphi));
                let (off, q) = joint_for(([0.0; 3], IDENT), pos, &IDENT, angle, child);
                Joint { holder: Holder::Column(strip), angle, off, q }
            } else {
                let pi = (ring - 1) * ns + strip;
                let parent = (panels[pi].loc, panels[pi].rot);
                let pb = *panels[pi].bricks.iter().find(|b| b.kind == Kind::Bearing).expect("parent bearing");
                let pos = [pb.pos.0 as f64, pb.pos.1 as f64, pb.pos.2 as f64];
                let rel = mm(&tr(&parent.1), &child.1);
                let delta = rel[0][2].atan2(rel[0][0]).to_degrees();
                let angle = wrap_deg(-delta);
                let (off, q) = joint_for(parent, pos, &D_HINGE, angle, child);
                Joint { holder: Holder::Panel(pi), angle, off, q }
            };
            panels[idx].joint = Some(joint);
        }
    }
    if max_err > 2.0 + 1e-6 {
        return Err(format!("internal: panels overhang their true shape by {max_err:.1} units"));
    }
    if panels.len() > 3000 {
        warnings.push(format!("{} entities is a lot; the game may be slow with this many.", panels.len()));
    }
    Ok(ShellResult { panels, main, column, outline: line, rings_at: rp, warnings, inside_sign })
}

// ------------------------------------------------------------------ check

#[derive(Debug, Default)]
pub struct ShellCheck {
    pub joint_error: f64,
    pub overlaps: usize,
    pub bad_sizes: usize,
    pub missing_bearings: usize,
}

impl ShellCheck {
    pub fn ok(&self) -> bool {
        self.joint_error < 0.01 && self.overlaps == 0 && self.bad_sizes == 0 && self.missing_bearings == 0
    }
}

fn boxes_overlap(a: &Pb, b: &Pb) -> bool {
    let (ha, hb) = (a.half(), b.half());
    (a.pos.0 - b.pos.0).abs() < ha.0 + hb.0 && (a.pos.1 - b.pos.1).abs() < ha.1 + hb.1 && (a.pos.2 - b.pos.2).abs() < ha.2 + hb.2
}

/// Wedges fill only half their box, but on a panel they sit beside the
/// rectangle, never on top, so a box test is exact here.
fn grid_overlaps(v: &[Pb]) -> usize {
    let mut n = 0;
    for i in 0..v.len() {
        for j in i + 1..v.len() {
            if boxes_overlap(&v[i], &v[j]) {
                n += 1;
            }
        }
    }
    n
}

/// Replays every joint with the game's rule and checks each panel lands where
/// the maths says; also checks overlaps and brick sizes.
pub fn check_shell(r: &ShellResult) -> ShellCheck {
    let mut c = ShellCheck::default();
    c.overlaps += grid_overlaps(&r.main);
    for p in &r.panels {
        c.overlaps += grid_overlaps(&p.bricks);
        for b in &p.bricks {
            let h = b.half();
            if b.kind != Kind::Bearing && (h.0 < 1 || h.1 < 1 || h.2 < 1 || h.0 > MAX_HALF || h.1 > MAX_HALF || h.2 > MAX_HALF) {
                c.bad_sizes += 1;
            }
        }
        if let Some(j) = &p.joint {
            let (parent, pos, d) = match j.holder {
                Holder::Column(k) => {
                    let Some(&bi) = r.column.get(k) else {
                        c.missing_bearings += 1;
                        continue;
                    };
                    let b = r.main[bi];
                    (([0.0; 3], IDENT), [b.pos.0 as f64, b.pos.1 as f64, b.pos.2 as f64], IDENT)
                }
                Holder::Panel(i) | Holder::PanelK(i, _) => {
                    let k = if let Holder::PanelK(_, k) = j.holder { k } else { 0 };
                    let q = &r.panels[i];
                    match q.bricks.iter().filter(|b| b.kind == Kind::Bearing).nth(k) {
                        Some(b) => ((q.loc, q.rot), [b.pos.0 as f64, b.pos.1 as f64, b.pos.2 as f64], d_of(b.ori)),
                        None => {
                            c.missing_bearings += 1;
                            continue;
                        }
                    }
                }
            };
            // what the game will do, using the rounded values it is given
            let qf = mat_from_quat(quat_from(&j.q));
            let (loc, rot) = joint_child(parent, pos, &d, j.angle as f32 as f64, j.off.map(|x| x as f32 as f64), &qf);
            let e = norm(sub(loc, p.loc));
            let mut er = 0.0f64;
            for a in 0..3 {
                for b in 0..3 {
                    er = er.max((rot[a][b] - p.rot[a][b]).abs());
                }
            }
            c.joint_error = c.joint_error.max(e).max(er * 100.0);
        }
    }
    c
}

// ------------------------------------------------------------------ world

/// Rough world bounds of everything (game units, before centring).
pub fn bounds(r: &ShellResult) -> (V, V) {
    let mut lo = [f64::MAX; 3];
    let mut hi = [f64::MIN; 3];
    let mut grow = |c: V, e: V| {
        for k in 0..3 {
            lo[k] = lo[k].min(c[k] - e[k]);
            hi[k] = hi[k].max(c[k] + e[k]);
        }
    };
    for b in &r.main {
        let h = b.half();
        grow([b.pos.0 as f64, b.pos.1 as f64, b.pos.2 as f64], [h.0 as f64, h.1 as f64, h.2 as f64]);
    }
    for p in &r.panels {
        let m = &p.rot;
        for b in &p.bricks {
            let h = b.half();
            let c = add(p.loc, mv(m, [b.pos.0 as f64, b.pos.1 as f64, b.pos.2 as f64]));
            let e = [
                m[0][0].abs() * h.0 as f64 + m[0][1].abs() * h.1 as f64 + m[0][2].abs() * h.2 as f64,
                m[1][0].abs() * h.0 as f64 + m[1][1].abs() * h.1 as f64 + m[1][2].abs() * h.2 as f64,
                m[2][0].abs() * h.0 as f64 + m[2][1].abs() * h.1 as f64 + m[2][2].abs() * h.2 as f64,
            ];
            grow(c, e);
        }
    }
    (lo, hi)
}

fn q4(q: [f64; 4]) -> Quat4f {
    Quat4f { x: q[0] as f32, y: q[1] as f32, z: q[2] as f32, w: q[3] as f32 }
}

pub fn shell_world(s: &ShellSettings, r: &ShellResult, name: &str) -> World {
    let (lo, hi) = bounds(r);
    let c = [
        ((lo[0] + hi[0]) / 2.0).round() as i32,
        ((lo[1] + hi[1]) / 2.0).round() as i32,
        ((lo[2] + hi[2]) / 2.0).round() as i32,
    ];
    let mut w = World::new();
    w.register_all_components();
    let mat = brdb::BString::from(s.material.clone());
    let colour = |i: usize| -> (u8, u8, u8) {
        let k = s.colours[i % s.colours.len()];
        (k[0], k[1], k[2])
    };
    let hidden = brdb::Collision {
        player: false,
        player1: None,
        player2: None,
        player3: None,
        weapon: false,
        interact: false,
        tool: false,
        physics: false,
    };
    let to_brick = |b: &Pb, col: (u8, u8, u8), shift: (i32, i32, i32)| -> Brick {
        let dir = [Direction::XPositive, Direction::XNegative, Direction::YPositive, Direction::YNegative, Direction::ZPositive, Direction::ZNegative]
            [(b.ori >> 2) as usize];
        let rot = [Rotation::Deg0, Rotation::Deg90, Rotation::Deg180, Rotation::Deg270][(b.ori & 3) as usize];
        let asset = match b.kind {
            Kind::Bearing => BrickType::Basic(brdb::BString::str(BEARING)),
            k => BrickType::Procedural {
                asset: brdb::BString::str(match k {
                    Kind::Wedge => MICRO_WEDGE,
                    Kind::SideWedge => SIDE_WEDGE,
                    _ => MICRO_BRICK,
                }),
                size: BrickSize { x: b.size.0 as u16, y: b.size.1 as u16, z: b.size.2 as u16 },
            },
        };
        let hide = s.hide_bearings && b.kind == Kind::Bearing;
        Brick {
            asset,
            position: Position::new(b.pos.0 - shift.0, b.pos.1 - shift.1, b.pos.2 - shift.2),
            direction: dir,
            rotation: rot,
            color: col.into(),
            material: mat.clone(),
            material_intensity: s.intensity,
            visible: !hide,
            collision: if hide { hidden } else { Default::default() },
            ..Default::default()
        }
    };
    let bearing_comp = |b: Brick, angle: f64| -> (Brick, usize) {
        b.with_component(LiteralComponent::new("Component_Internal_RigidBearing").with_data([
            ("TargetAngle", Box::new(angle as f32) as Box<dyn AsBrdbValue>),
            ("bAnglesArePercentages", Box::new(false)),
            ("bReversed", Box::new(false)),
            ("CurrentAngle", Box::new(angle as f32)),
            ("InterpMode", Box::new(2u8)),
            ("SmoothTime", Box::new(0.25f32)),
            ("TopSpeed", Box::new(0.0f32)),
        ]))
        .with_id_split()
    };
    let ns = s.strips as usize;
    // angles each bearing must hold: column bearing k -> ring 0 of strip k,
    // panel i's bearing -> the panel after it in the strip
    let mut held_angle: std::collections::HashMap<(usize, usize), f64> = Default::default();
    let mut column_angle: Vec<f64> = vec![0.0; r.column.len()];
    for p in &r.panels {
        if let Some(j) = &p.joint {
            match j.holder {
                Holder::Column(k) => column_angle[k] = j.angle,
                Holder::Panel(i) => {
                    held_angle.insert((i, 0), j.angle);
                }
                Holder::PanelK(i, k) => {
                    held_angle.insert((i, k), j.angle);
                }
            }
        }
    }
    // main grid
    let mut column_ids: Vec<usize> = vec![0; r.column.len()];
    for (bi, b) in r.main.iter().enumerate() {
        let mut br = to_brick(b, colour(0), (c[0], c[1], c[2]));
        if b.kind != Kind::Bearing && s.hide_bearings {
            br.visible = false;
            br.collision = hidden;
        }
        if let Some(k) = r.column.iter().position(|&x| x == bi) {
            let (br, id) = bearing_comp(br, column_angle[k]);
            column_ids[k] = id;
            w.add_brick(br);
        } else {
            w.add_brick(br);
        }
    }
    // panels
    let mut ent_ids: Vec<usize> = vec![];
    let mut hinge_ids: Vec<Vec<usize>> = vec![vec![]; r.panels.len()];
    for (pi, p) in r.panels.iter().enumerate() {
        let col = colour(p.colour);
        let mut bricks = vec![];
        for b in &p.bricks {
            let br = to_brick(b, col, (0, 0, 0));
            if b.kind == Kind::Bearing {
                let k = hinge_ids[pi].len();
                let (br, id) = bearing_comp(br, held_angle.get(&(pi, k)).copied().unwrap_or(0.0));
                hinge_ids[pi].push(id);
                bricks.push(br);
            } else {
                bricks.push(br);
            }
        }
        let id = Brick::next_id();
        ent_ids.push(id);
        w.add_brick_grid(
            Entity {
                id: Some(id),
                frozen: true,
                location: Vector3f {
                    x: (p.loc[0] - c[0] as f64) as f32,
                    y: (p.loc[1] - c[1] as f64) as f32,
                    z: (p.loc[2] - c[2] as f64) as f32,
                },
                rotation: q4(quat_from(&p.rot)),
                ..Default::default()
            },
            bricks,
        );
    }
    let _ = ns;
    for (pi, p) in r.panels.iter().enumerate() {
        if let Some(j) = &p.joint {
            let bid = match j.holder {
                Holder::Column(k) => column_ids[k],
                Holder::Panel(i) => hinge_ids[i][0],
                Holder::PanelK(i, k) => hinge_ids[i][k],
            };
            w.add_joint(
                bid,
                ent_ids[pi],
                Vector3f { x: j.off[0] as f32, y: j.off[1] as f32, z: j.off[2] as f32 },
                q4(quat_from(&j.q)),
            );
        }
    }
    w.meta.bundle.name = name.to_string();
    w.meta.bundle.description = "Made with Circle Generator (Shell tab)".into();
    w.meta.bundle.game_version = "CL16507".into();
    w.make_prefab();
    // prefab size from everything (make_prefab only measures the main grid)
    let hx = (lo[0] - c[0] as f64).abs().max(hi[0] - c[0] as f64).ceil() as i32;
    let hy = (lo[1] - c[1] as f64).abs().max(hi[1] - c[1] as f64).ceil() as i32;
    let hz = (lo[2] - c[2] as f64).abs().max(hi[2] - c[2] as f64).ceil() as i32;
    let mut pj = brdb::PrefabJson::from_bounds(Position::new(-hx, -hy, -hz), Position::new(hx, hy, hz));
    if let Some(old) = &w.meta.prefab {
        pj.added_global_grid_offset = old.added_global_grid_offset.clone();
    }
    w.meta.prefab = Some(pj);
    w
}

pub fn shell_bytes(s: &ShellSettings, name: &str) -> Result<(ShellResult, Vec<u8>), String> {
    let r = build_shell(s)?;
    let chk = check_shell(&r);
    if !chk.ok() {
        return Err(format!("Self-check failed ({chk:?}). Nothing was saved. Please send this settings combination to Claude."));
    }
    let w = shell_world(s, &r, name);
    let bytes = w.to_brz_vec().map_err(|e| e.to_string())?;
    Ok((r, bytes))
}
