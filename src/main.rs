//! Circle Generator: the window. All the shape-making lives in the library
//! (src/lib.rs + src/geometry.rs); this file only draws controls and the
//! preview, and saves / copies the result.
#![cfg_attr(windows, windows_subsystem = "windows")]

use circlegen::cone::{self, ConeFill, ConeResult, ConeSettings, ConeType, Levels};
use circlegen::icosphere;
use circlegen::shell::{self, BearingSide, ColourMode, Curve, IcoBrick, Movement, Seam, ShellResult, ShellSettings, Spacing, SphereInput};
use circlegen::{build, check, cone_world, to_world, BrickStyle, Fill, Kind, Layout, Settings, Shape};
use eframe::egui::{self, Color32, CornerRadius, Pos2, RichText, Stroke, Vec2};

// ---- colours (matched to heightmap2brz's brz tools) ----
const BG: Color32 = Color32::from_rgb(0x16, 0x23, 0x3a);
const CARD: Color32 = Color32::from_rgb(0x46, 0x5a, 0x80);
const FIELD: Color32 = Color32::from_rgb(0x1d, 0x2a, 0x43);
const ACCENT: Color32 = Color32::from_rgb(0x3d, 0x8b, 0xfd);
const GREEN: Color32 = Color32::from_rgb(0x3d, 0xbb, 0x6b);
const PREVIEW: Color32 = Color32::from_rgb(0x0d, 0x16, 0x26);
const FILLER: Color32 = Color32::from_rgb(0xdf, 0xe4, 0xef);
const WEDGE: Color32 = Color32::from_rgb(0x7b, 0x9b, 0xe6);
const OUTLINE: Color32 = Color32::from_rgb(0x30, 0x38, 0x4c);
const TRUE_CIRCLE: Color32 = Color32::from_rgb(0xf2, 0xa7, 0xb7);
const SOFT: Color32 = Color32::from_rgb(0xc5, 0xd0, 0xe4);
const WARN: Color32 = Color32::from_rgb(0xff, 0xc8, 0x57);
const ERR: Color32 = Color32::from_rgb(0xff, 0x7b, 0x7b);

const MATERIALS: [(&str, &str); 6] = [
    ("Plastic", "BMC_Plastic"),
    ("Glass", "BMC_Glass"),
    ("Translucent", "BMC_TranslucentPlastic"),
    ("Glow", "BMC_Glow"),
    ("Metallic", "BMC_Metallic"),
    ("Hologram", "BMC_Hologram"),
];

/// Slider tops: 800 studs (the longest single brick). Larger values can be typed.
const SLIDER_MAX_MICRO: i32 = 4000;
const SLIDER_MAX_STUD: i32 = 800;
const TYPE_MAX_MICRO: i32 = 20000;
const TYPE_MAX_STUD: i32 = 4000;

#[derive(Clone, Copy, PartialEq)]
enum LogKind {
    Info,
    Ok,
    Warn,
    Err,
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Circle,
    Cone,
    Shell,
    Ico,
}

struct App {
    tab: Tab,
    // ---- shell tab ----
    s_set: ShellSettings,
    // curve editor
    s_prev_curve: Curve,
    s_sel: Option<usize>,
    s_drag: Option<usize>,
    s_snap: bool,
    s_snap_stud: bool,
    s_lock: bool,
    s_ebox: Option<(f64, f64, f64, f64)>,
    s_best: Option<((u32, IcoBrick), Vec<(f64, f64)>)>,
    s_key: String,
    s_built: Option<Result<ShellResult, String>>,
    s_view: (f32, f32),
    s_mat: usize,
    // ---- cone tab ----
    c_shape: Shape,
    c_type: ConeType,
    c_levels: Levels,
    c_oval: bool,
    c_bw: i32,
    c_bl: i32,
    c_tw: i32,
    c_tl: i32,
    c_h: i32,
    c_block: i32,
    c_band: i32,
    c_fill: ConeFill,
    c_flip: bool,
    c_key: String,
    c_built: Option<Result<ConeResult, String>>,
    // ---- circle tab ----
    shape: Shape,
    kind: Kind,
    inverse: bool,
    /// fill choice for: solid, solid inverse, doughnut, doughnut inverse
    fills: [Fill; 4],
    style: BrickStyle,
    /// sizes in the current style's units (micro bricks for Micro;
    /// studs / plates / studs for stud styles)
    oval: bool,
    /// width/length of the reference outline: the outside, or the hole
    /// when a doughnut is set by its hole size
    width: i32,
    length: i32,
    ring_by_hole: bool,
    band: i32,
    height: i32,
    wall: i32,
    color: [u8; 3],
    hex: String,
    material: usize,
    file_name: String,
    name_edited: bool,
    log: Vec<(LogKind, String)>,
    // cached result
    key: String,
    built: Option<Result<circlegen::Built, String>>,
    confirm_replace: Option<std::path::PathBuf>,
    always_on_top: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        style_ui(&cc.egui_ctx);
        let mut a = App {
            tab: Tab::Circle,
            s_set: ShellSettings::default(),
            s_prev_curve: Curve::Dish,
            s_sel: None,
            s_drag: None,
            s_snap: true,
            s_snap_stud: false,
            s_lock: true,
            s_ebox: None,
            s_best: None,
            s_key: String::new(),
            s_built: None,
            s_view: (35.0, 25.0),
            s_mat: 0,
            c_shape: Shape::Full,
            c_type: ConeType::Solid,
            c_levels: Levels::Single,
            c_oval: false,
            c_bw: 40,
            c_bl: 40,
            c_tw: 38,
            c_tl: 38,
            c_h: 4,
            c_block: 60,
            c_band: 15,
            c_fill: ConeFill::LargeBlocks,
            c_flip: false,
            c_key: String::new(),
            c_built: None,
            shape: Shape::Full,
            kind: Kind::Solid,
            inverse: false,
            fills: [Fill::Blocks; 4],
            style: BrickStyle::Micro,
            oval: false,
            width: 80,
            length: 80,
            ring_by_hole: false,
            band: 10,
            height: 18,
            wall: 5,
            color: [255, 255, 255],
            hex: "#FFFFFF".into(),
            material: 0,
            file_name: String::new(),
            name_edited: false,
            log: vec![(LogKind::Info, "Ready. Choose a circle, then Save or Copy.".into())],
            key: String::new(),
            built: None,
            confirm_replace: None,
            always_on_top: false,
        };
        match std::env::var("CG_TAB").as_deref() {
            Ok("cone") => a.tab = Tab::Cone,
            Ok("shell") => a.tab = Tab::Shell,
            Ok("icosphere") => a.tab = Tab::Ico,
            Ok("custom") => {
                a.tab = Tab::Shell;
                a.s_set.curve = Curve::Custom;
                a.s_sel = Some(4);
            }
            _ => {}
        }
        a.file_name = a.auto_name();
        a
    }

    fn fill_slot(&self) -> Option<usize> {
        match (self.kind, self.inverse) {
            (Kind::Solid, false) => Some(0),
            (Kind::Solid, true) => Some(1),
            (Kind::Ring, false) => Some(2),
            (Kind::Ring, true) => Some(3),
            _ => None,
        }
    }

    fn fill(&self) -> Fill {
        self.fill_slot().map_or(Fill::Strips, |i| self.fills[i])
    }

    /// (outer width, outer length, hole width, hole length) in cells
    fn sizes(&self) -> (i32, i32, i32, i32) {
        let (w, l) = (self.width, if self.oval { self.length } else { self.width });
        if self.kind != Kind::Ring {
            return (w, l, 0, 0);
        }
        let b = 2 * self.band;
        if self.ring_by_hole { (w + b, l + b, w, l) } else { (w, l, w - b, l - b) }
    }

    fn settings(&self) -> Settings {
        Settings {
            layout: Layout {
                shape: self.shape,
                kind: self.kind,
                inverse: self.inverse,
                fill: self.fill(),
                width: self.sizes().0,
                length: self.sizes().1,
                hole_w: self.sizes().2,
                hole_l: self.sizes().3,
                wall: self.wall,
            },
            style: self.style,
            height: self.height,
            color: (self.color[0], self.color[1], self.color[2]),
            material: MATERIALS[self.material].1.into(),
            name: self.file_name.clone(),
        }
    }

    fn unit(&self) -> &'static str {
        if self.style.is_stud() { "studs" } else { "M" }
    }

    fn auto_name(&self) -> String {
        if matches!(self.tab, Tab::Shell | Tab::Ico) {
            return self.shell_name();
        }
        if self.tab == Tab::Cone {
            return self.cone_name();
        }
        let shape = match self.shape {
            Shape::Full => "Full",
            Shape::Half => "Half",
            Shape::Quarter => "Quarter",
        };
        let ty = match (self.kind, self.inverse) {
            (Kind::Solid, false) => "Solid",
            (Kind::Hollow, false) => "Hollow",
            (Kind::Ring, false) => "Doughnut",
            (Kind::Solid, true) => "SolidInverse",
            (Kind::Hollow, true) => "HollowInverse",
            (Kind::Ring, true) => "DoughnutInverse",
        };
        let fill = match (self.kind, self.inverse, self.fill()) {
            (Kind::Hollow, _, _) => "",
            (_, _, Fill::Strips) => "_Strips",
            (Kind::Solid, false, _) => if self.oval { "_RectangleCentre" } else { "_SquareCentre" },
            (Kind::Solid, true, _) => "_CornerBlocks",
            (Kind::Ring, false, _) => "_FourWay",
            (Kind::Ring, true, _) => "_Blocks",
        };
        let style = match self.style {
            BrickStyle::Micro => "Micro",
            BrickStyle::Default => "Default",
            BrickStyle::Tile => "Tile",
            BrickStyle::Smooth => "Smooth",
        };
        let (u, hu) = if self.style.is_stud() { ("studs", "plates") } else { ("M", "M") };
        let (ow, ol, hw, hl) = self.sizes();
        let size = |w: i32, l: i32| if w == l { format!("{w}{u}") } else { format!("{w}x{l}{u}") };
        let mut d = size(ow, ol);
        if self.kind == Kind::Ring {
            d = format!("{d}_hole{}", size(hw, hl));
        }
        let h = format!("{}{hu}", self.height);
        let base = if self.oval { "Oval" } else { "Circle" };
        format!("{base}_{shape}_{ty}{fill}_{style}_{d}_{h}")
    }

    fn set_style(&mut self, s: BrickStyle) {
        if s.is_stud() == self.style.is_stud() {
            self.style = s;
            return;
        }
        if s.is_stud() {
            // micro bricks -> studs / plates
            self.width = ((self.width as f64 / 5.0).round() as i32).max(2);
            self.length = ((self.length as f64 / 5.0).round() as i32).max(2);
            self.band = ((self.band as f64 / 5.0).round() as i32).max(1);
            self.height = ((self.height as f64 / 2.0).round() as i32).max(1);
            self.wall = ((self.wall as f64 / 5.0).round() as i32).max(1);
        } else {
            self.width *= 5;
            self.length *= 5;
            self.band *= 5;
            self.height *= 2;
            self.wall *= 5;
        }
        self.style = s;
    }

    fn log(&mut self, k: LogKind, m: impl Into<String>) {
        self.log.push((k, m.into()));
        if self.log.len() > 50 {
            self.log.remove(0);
        }
    }

    fn rebuild_if_needed(&mut self) {
        // the Icosphere tab and the Shell tab share one set of settings
        if self.tab == Tab::Ico && self.s_set.curve != Curve::Icosphere {
            self.s_prev_curve = self.s_set.curve;
            self.s_set.curve = Curve::Icosphere;
        } else if self.tab == Tab::Shell && self.s_set.curve == Curve::Icosphere {
            self.s_set.curve = self.s_prev_curve;
        }
        if matches!(self.tab, Tab::Shell | Tab::Ico) {
            let s = self.shell_settings();
            let key = format!("{s:?}");
            if key != self.s_key {
                self.s_key = key;
                self.s_built = Some(shell::build_shell(&s));
                if !self.name_edited {
                    self.file_name = self.auto_name();
                }
            }
            return;
        }
        if self.tab == Tab::Cone {
            let c = self.cone_settings();
            let key = format!("{c:?}");
            if key != self.c_key {
                self.c_key = key;
                self.c_built = Some(cone::build_cone(&c));
                if !self.name_edited {
                    self.file_name = self.auto_name();
                }
            }
            return;
        }
        let s = self.settings();
        let key = format!("{:?}{:?}{}{:?}{}{}", s.layout, s.style, s.height, s.color, s.material, self.oval);
        if key != self.key {
            self.key = key;
            self.built = Some(build(&s));
            if !self.name_edited {
                self.file_name = self.auto_name();
            }
        }
    }

    /// Full gap / overlap check before anything leaves the program.
    fn checked_bytes(&mut self) -> Option<(Vec<u8>, usize)> {
        if matches!(self.tab, Tab::Shell | Tab::Ico) {
            return self.shell_bytes();
        }
        if self.tab == Tab::Cone {
            return self.cone_bytes();
        }
        let mut s = self.settings();
        s.name = self.file_name.trim().to_string();
        let built = match build(&s) {
            Ok(b) => b,
            Err(e) => {
                self.log(LogKind::Err, e);
                return None;
            }
        };
        let ext = (s.layout.width.max(s.layout.length) + 2 * s.layout.wall + 4) as f64;
        let step = (2.0 * ext / 1600.0).max(0.5);
        let r = check::check_with_step(&s.layout, &built.pieces, step);
        if !r.ok() {
            self.log(LogKind::Err, format!("Self-check failed ({r:?}). Nothing was saved. Please send this settings combination to Claude."));
            return None;
        }
        let n = built.bricks.len();
        let w = to_world(&s, &built);
        match w.to_brz_vec() {
            Ok(b) => Some((b, n)),
            Err(e) => {
                self.log(LogKind::Err, format!("Could not create the prefab: {e}"));
                None
            }
        }
    }

    fn clean_name(&self) -> String {
        let n: String = self
            .file_name
            .trim()
            .chars()
            .map(|c| if "<>:\"/\\|?*".contains(c) || c.is_control() { '_' } else { c })
            .collect();
        let n = n.trim_end_matches(".brz").to_string();
        if n.is_empty() { "Circle".into() } else { n }
    }

    fn prefab_dir() -> Option<std::path::PathBuf> {
        dirs::data_local_dir().map(|d| d.join("Brickadia").join("Saved").join("Prefabs"))
    }

    fn save(&mut self, replace: bool) {
        let Some(dir) = Self::prefab_dir() else {
            self.log(LogKind::Err, "Could not find your AppData\\Local folder.");
            return;
        };
        let path = dir.join(format!("{}.brz", self.clean_name()));
        if path.exists() && !replace {
            self.confirm_replace = Some(path);
            return;
        }
        let Some((bytes, n)) = self.checked_bytes() else { return };
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, bytes)) {
            self.log(LogKind::Err, format!("Could not save: {e}"));
            return;
        }
        self.log(LogKind::Ok, format!("Saved {n} bricks to {} (self-check passed).", path.display()));
    }

    fn copy(&mut self) {
        let Some((bytes, n)) = self.checked_bytes() else { return };
        let dir = std::env::temp_dir().join("CircleGenerator");
        let path = dir.join(format!("{}.brz", self.clean_name()));
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, bytes)) {
            self.log(LogKind::Err, format!("Could not prepare the copy: {e}"));
            return;
        }
        match copy_file_to_clipboard(&path) {
            Ok(()) => self.log(LogKind::Ok, format!("Copied {n} bricks. In game, paste with Ctrl+V (self-check passed).")),
            Err(e) => self.log(LogKind::Err, e),
        }
    }
}

#[cfg(windows)]
fn copy_file_to_clipboard(path: &std::path::Path) -> Result<(), String> {
    let p = path.to_string_lossy().to_string();
    clipboard_win::raw::open().map_err(|e| format!("Could not open the clipboard: {e}"))?;
    let set = clipboard_win::raw::set_file_list(&[p]).map_err(|e| format!("Could not copy: {e}"));
    let _ = clipboard_win::raw::close();
    set
}

#[cfg(not(windows))]
fn copy_file_to_clipboard(path: &std::path::Path) -> Result<(), String> {
    Err(format!("Copy only works on Windows. The file is at {}", path.display()))
}

fn style_ui(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("dv".into(), std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/DejaVuSans.ttf"))));
    fonts.font_data.insert("dvb".into(), std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/DejaVuSans-Bold.ttf"))));
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "dv".into());
    fonts.families.insert(egui::FontFamily::Name("bold".into()), vec!["dvb".into(), "dv".into()]);
    ctx.set_fonts(fonts);
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = BG;
    v.extreme_bg_color = FIELD;
    v.selection.bg_fill = ACCENT;
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x2b, 0x3a, 0x57);
    v.widgets.inactive.bg_fill = Color32::from_rgb(0x2b, 0x3a, 0x57);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x36, 0x49, 0x6c);
    v.widgets.active.weak_bg_fill = ACCENT;
    v.slider_trailing_fill = true;
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(8.0, 7.0);
        s.spacing.button_padding = Vec2::new(10.0, 5.0);
        s.spacing.slider_width = 120.0;
        s.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        s.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
    });
}

fn bold(t: impl Into<String>, size: f32) -> RichText {
    RichText::new(t).family(egui::FontFamily::Name("bold".into())).size(size)
}

fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let w = ui.available_width() - 28.0;
    egui::Frame::new()
        .fill(CARD)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(w);
            add(ui)
        })
        .inner
}

fn label(ui: &mut egui::Ui, t: &str) {
    ui.allocate_ui_with_layout(Vec2::new(108.0, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_width(108.0);
        ui.label(bold(t, 13.5).color(Color32::WHITE));
    });
}

/// Segmented buttons. Returns true if changed.
fn seg<T: PartialEq + Copy>(ui: &mut egui::Ui, v: &mut T, opts: &[(T, &str)]) -> bool {
    let mut changed = false;
    egui::Frame::new()
        .fill(Color32::from_rgb(0x2b, 0x3a, 0x57))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::same(3))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            ui.horizontal(|ui| {
                for (val, text) in opts {
                    let on = *v == *val;
                    let b = egui::Button::new(bold(*text, 13.5).color(if on { Color32::WHITE } else { SOFT }))
                        .fill(if on { ACCENT } else { Color32::TRANSPARENT })
                        .corner_radius(CornerRadius::same(6));
                    if ui.add(b).clicked() && !on {
                        *v = *val;
                        changed = true;
                    }
                }
            });
        });
    changed
}

fn studs(x: f64) -> String {
    let n = fmt_num(x);
    if n == "1" { "1 stud".into() } else { format!("{n} studs") }
}

fn fmt_num(x: f64) -> String {
    let s = format!("{:.2}", x);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// Up / Down arrows change a focused number box by `step`, Shift + arrow by 10.
fn arrow_keys(ui: &mut egui::Ui, v: &mut i32, step: i32, min: i32, max: i32) {
    let id = ui.next_auto_id();
    if ui.memory(|m| m.has_focus(id)) {
        // egui matches arrow keys whether or not Shift is held, so read Shift separately
        let (shift, up, dn) = ui.input_mut(|i| {
            (
                i.modifiers.shift,
                i.count_and_consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) as i32,
                i.count_and_consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) as i32,
            )
        });
        let d = (up - dn) * if shift { 10 } else { step };
        if d != 0 {
            *v = (*v + d).clamp(min, max);
            ui.data_mut(|data| data.remove::<String>(id));
        }
    }
}

/// Number box with arrow keys.
fn num_box(ui: &mut egui::Ui, v: &mut i32, min: i32, max: i32, step: i32) {
    ui.allocate_ui_with_layout(Vec2::new(62.0, 22.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
        arrow_keys(ui, v, step, min, max);
        ui.add(egui::DragValue::new(v).range(min..=max).speed(0.2));
    });
}

/// Slider + type-in box. Slider stops at `slider_max`; the box goes to `type_max`.
fn size_row(ui: &mut egui::Ui, name: &str, v: &mut i32, min: i32, slider_max: i32, type_max: i32, note: String, enabled: bool) -> bool {
    size_row_s(ui, name, v, min, slider_max, type_max, note, enabled, 1)
}

fn size_row_s(ui: &mut egui::Ui, name: &str, v: &mut i32, min: i32, slider_max: i32, type_max: i32, note: String, enabled: bool, step: i32) -> bool {
    let before = *v;
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            label(ui, name);
            let mut sv = (*v).min(slider_max);
            if ui.add(egui::Slider::new(&mut sv, min..=slider_max).logarithmic(slider_max > 100).show_value(false)).changed() {
                *v = sv;
            }
            num_box(ui, v, min, type_max, step);
            ui.label(RichText::new(note).color(Color32::from_rgb(0xd6, 0xde, 0xf0)).size(12.5));
        });
    });
    *v != before
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.rebuild_if_needed();

        egui::TopBottomPanel::top("head")
            .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin { left: 20, right: 20, top: 14, bottom: 10 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(bold("Circle Generator", 26.0).color(Color32::WHITE));
                            ui.label(RichText::new(concat!("v", env!("CARGO_PKG_VERSION"))).color(Color32::from_rgb(0x8f, 0xa2, 0xc2)));
                        });
                        ui.label(bold("Smooth circles, cones, curved shells and icospheres, saved as Brickadia prefabs.", 13.0).color(SOFT));
                        ui.add_space(4.0);
                        if seg(ui, &mut self.tab, &[(Tab::Circle, "◯ Circle"), (Tab::Cone, "△ Cone"), (Tab::Shell, "◠ Shell"), (Tab::Ico, "⬡ Icosphere")]) && !self.name_edited {
                            self.key.clear();
                            self.c_key.clear();
                            self.s_key.clear();
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                        if ui.checkbox(&mut self.always_on_top, "Always on top").changed() {
                            let level = if self.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("log")
            .exact_height(46.0)
            .frame(egui::Frame::new().fill(Color32::BLACK).inner_margin(egui::Margin::symmetric(12, 6)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for (k, m) in &self.log {
                        let c = match k {
                            LogKind::Info => Color32::from_rgb(0xd7, 0xe3, 0xd7),
                            LogKind::Ok => Color32::from_rgb(0x7f, 0xe0, 0x8f),
                            LogKind::Warn => WARN,
                            LogKind::Err => ERR,
                        };
                        ui.label(RichText::new(format!("[INFO] {m}")).monospace().size(12.0).color(c));
                    }
                });
            });

        egui::SidePanel::left("controls")
            .exact_width(540.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin { left: 16, right: 8, top: 12, bottom: 12 }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin { left: 8, right: 16, top: 12, bottom: 12 }))
            .show(ctx, |ui| self.preview_panel(ui));

        // replace-file confirmation
        if let Some(path) = self.confirm_replace.clone() {
            let mut close = false;
            let mut go = false;
            egui::Modal::new(egui::Id::new("replace")).show(ctx, |ui| {
                ui.label(RichText::new("A prefab with this name already exists").strong().size(16.0));
                ui.label(path.display().to_string());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Replace it").clicked() {
                        go = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });
            if go {
                self.confirm_replace = None;
                self.save(true);
            } else if close {
                self.confirm_replace = None;
                self.log(LogKind::Info, "Not saved. Change the file name to keep both.");
            }
        }
    }
}

impl App {
    fn controls(&mut self, ui: &mut egui::Ui) {
        if matches!(self.tab, Tab::Shell | Tab::Ico) {
            self.shell_controls(ui);
            return;
        }
        if self.tab == Tab::Cone {
            self.cone_controls(ui);
            self.colour_card(ui);
            return;
        }
        card(ui, |ui| {
            ui.horizontal(|ui| {
                label(ui, "Shape");
                seg(ui, &mut self.shape, &[(Shape::Full, "● Full"), (Shape::Half, "◓ Half"), (Shape::Quarter, "◔ Quarter")]);
                ui.checkbox(&mut self.oval, bold("Oval", 13.5));
            });
            ui.horizontal(|ui| {
                label(ui, "Type");
                seg(ui, &mut self.kind, &[(Kind::Solid, "Solid"), (Kind::Hollow, "Hollow"), (Kind::Ring, "Doughnut")]);
                ui.checkbox(&mut self.inverse, bold("Inverse", 13.5));
            });
            ui.horizontal(|ui| {
                label(ui, "Fill");
                let centre = if self.oval { "Rectangle centre" } else { "Square centre" };
                match self.fill_slot() {
                    Some(i) => {
                        let other = match i {
                            0 => centre,
                            1 => "Corner blocks",
                            2 => "Four-way",
                            _ => "Blocks",
                        };
                        seg(ui, &mut self.fills[i], &[(Fill::Strips, "Strips"), (Fill::Blocks, other)]);
                    }
                    None => {
                        ui.label(RichText::new("Hollow shapes have no filler").italics().color(SOFT));
                    }
                }
            });
            ui.horizontal(|ui| {
                label(ui, "Brick type");
                let mut s = self.style;
                if seg(ui, &mut s, &[(BrickStyle::Micro, "Micro"), (BrickStyle::Default, "Default"), (BrickStyle::Tile, "Tile"), (BrickStyle::Smooth, "Smooth")]) {
                    self.set_style(s);
                }
            });
        });
        ui.add_space(4.0);

        let stud = self.style.is_stud();
        let (smax, tmax, dmin) = if stud { (SLIDER_MAX_STUD, TYPE_MAX_STUD, 2) } else { (SLIDER_MAX_MICRO, TYPE_MAX_MICRO, 6) };
        let note = |v: f64| if stud { format!("studs ({} M)", fmt_num(v * 5.0)) } else { format!("M ({})", studs(v / 5.0)) };
        let ring = self.kind == Kind::Ring;
        card(ui, |ui| {
            if ring {
                ui.horizontal(|ui| {
                    label(ui, "Set by");
                    seg(ui, &mut self.ring_by_hole, &[(false, "Outer size"), (true, "Hole size")]);
                });
            }
            let which = if !ring { "" } else if self.ring_by_hole { "Hole " } else { "Outer " };
            if !self.oval {
                let dlo = dmin.min(self.width.max(2));
                let n = note(self.width as f64);
                let st = if self.shape == Shape::Full { 1 } else { 2 };
                size_row_s(ui, if which == "Hole " { "Hole diameter" } else { "Diameter" }, &mut self.width, dlo, smax, tmax, n, true, st);
                // radius, linked
                ui.horizontal(|ui| {
                    label(ui, if which == "Hole " { "Hole radius" } else { "Radius" });
                    let mut r = self.width as f64 / 2.0;
                    let mut sr = r.min(smax as f64 / 2.0);
                    if ui.add(egui::Slider::new(&mut sr, (dmin as f64 / 2.0)..=(smax as f64 / 2.0)).logarithmic(true).show_value(false).step_by(0.5)).changed() {
                        self.width = (sr * 2.0).round() as i32;
                    }
                    let mut dia = self.width;
                    ui.allocate_ui_with_layout(Vec2::new(62.0, 22.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                        arrow_keys(ui, &mut dia, 1, 2, tmax);
                        if dia != self.width {
                            r = dia as f64 / 2.0;
                        }
                        if ui.add(egui::DragValue::new(&mut r).range(1.0..=(tmax as f64 / 2.0)).speed(0.1).max_decimals(1).custom_formatter(|v, _| fmt_num(v))).changed() || dia != self.width {
                            self.width = (r * 2.0).round() as i32;
                        }
                    });
                    ui.label(RichText::new(note(self.width as f64 / 2.0)).color(Color32::from_rgb(0xd6, 0xde, 0xf0)).size(12.5));
                });
                ui.label(RichText::new("                              🔗 Diameter and radius are linked").size(11.5).color(Color32::from_rgb(0x9f, 0xb6, 0xdc)));
                self.length = self.width;
            } else {
                let lo = dmin.min(self.width.max(2));
                let n = note(self.width as f64);
                size_row(ui, match which { "Hole " => "Hole width", "Outer " => "Outer width", _ => "Width" }, &mut self.width, lo, smax, tmax, n, true);
                let lo = dmin.min(self.length.max(2));
                let n = note(self.length as f64);
                size_row(ui, match which { "Hole " => "Hole length", "Outer " => "Outer length", _ => "Length" }, &mut self.length, lo, smax, tmax, n, true);
                ui.label(RichText::new("                              Half ovals keep the top half (swap the sizes to flip)").size(11.5).color(Color32::from_rgb(0x9f, 0xb6, 0xdc)));
            }
            if ring {
                let n = if stud { format!("studs ({} M)", self.band * 5) } else { format!("M ({})", studs(self.band as f64 / 5.0)) };
                let bmax = if stud { 200 } else { 1000 };
                size_row(ui, "Band", &mut self.band, 1, bmax, tmax / 2, n, true);
                let (ow, ol, hw, hl) = self.sizes();
                let u = if stud { "studs" } else { "M" };
                let st = |v: i32| if stud { format!("{v} studs") } else { format!("{v} M ({})", studs(v as f64 / 5.0)) };
                let _ = u;
                let txt = if self.ring_by_hole {
                    if self.oval { format!("Outer size: {} × {}", st(ow), st(ol)) } else { format!("Outer diameter: {}", st(ow)) }
                } else if self.oval {
                    format!("Hole: {} × {}", st(hw), st(hl))
                } else {
                    format!("Hole diameter: {}", st(hw))
                };
                ui.label(RichText::new(format!("                              {txt}")).size(12.5).strong().color(Color32::WHITE));
                if !stud && self.band < 2 {
                    ui.label(RichText::new("A 1 M band only works at some sizes; 2 M or more always works.").size(12.0).color(WARN));
                }
            }
            let (ow, ol, _, _) = self.sizes();
            if ow.max(ol) > smax {
                ui.label(RichText::new(format!("Bigger than one brick ({smax} {}), so long strips will be split into equal pieces.", self.unit())).size(12.0).color(WARN));
            }
            // halves and quarters need even sizes on the cut
            let odd_len = self.shape != Shape::Full && self.length % 2 != 0;
            let odd_w = self.shape == Shape::Quarter && self.width % 2 != 0;
            if odd_len || odd_w {
                let what = if !self.oval { "diameter" } else if odd_len && odd_w { "width and length" } else if odd_len { "length" } else { "width" };
                ui.label(RichText::new(format!("Half and quarter shapes need an even {what}: the cut runs through the centre, and an odd size would mean cutting a brick in half.")).size(12.0).color(WARN));
                ui.horizontal(|ui| {
                    if !self.oval {
                        if ui.button(format!("Use {}", self.width - 1)).clicked() { self.width -= 1; self.length = self.width; }
                        if ui.button(format!("Use {}", self.width + 1)).clicked() { self.width += 1; self.length = self.width; }
                    } else {
                        if odd_w && ui.button(format!("Width {}", self.width + 1)).clicked() { self.width += 1; }
                        if odd_len && ui.button(format!("Length {}", self.length + 1)).clicked() { self.length += 1; }
                    }
                });
            }
            let h_note = if stud {
                format!("plates ({} bricks)", fmt_num(self.height as f64 / 3.0))
            } else {
                format!("M ({} bricks / {} plates)", fmt_num(self.height as f64 / 6.0), fmt_num(self.height as f64 / 2.0))
            };
            let (hmax, htype) = if stud { (2000, 2000) } else { (4000, 4000) }; // tallest single brick
            size_row(ui, "Height", &mut self.height, 1, hmax, htype, h_note, true);
            let inv_solid = self.kind != Kind::Hollow && self.inverse;
            let w_note = if stud {
                format!("studs{}", if inv_solid { " · thinnest point" } else { " · inverse only" })
            } else {
                format!("M ({}){}", studs(self.wall as f64 / 5.0), if inv_solid { "" } else { " · inverse only" })
            };
            let (wmax, wtype) = if stud { (40, 800) } else { (200, 4000) };
            size_row(ui, "Wall", &mut self.wall, 1, wmax, wtype, w_note, inv_solid);
        });
        ui.add_space(4.0);

        self.colour_card(ui);
    }

    fn colour_card(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                label(ui, "Colour");
                if egui::color_picker::color_edit_button_srgb(ui, &mut self.color).changed() {
                    self.hex = format!("#{:02X}{:02X}{:02X}", self.color[0], self.color[1], self.color[2]);
                }
                let r = ui.add_sized([90.0, 22.0], egui::TextEdit::singleline(&mut self.hex));
                if r.changed() {
                    let h = self.hex.trim().trim_start_matches('#');
                    if h.len() == 6 {
                        if let Ok(v) = u32::from_str_radix(h, 16) {
                            self.color = [(v >> 16) as u8, (v >> 8) as u8, v as u8];
                        }
                    }
                }
            });
            ui.horizontal(|ui| {
                label(ui, "Material");
                egui::ComboBox::from_id_salt("material")
                    .width(150.0)
                    .selected_text(MATERIALS[self.material].0)
                    .show_ui(ui, |ui| {
                        for (i, (n, _)) in MATERIALS.iter().enumerate() {
                            ui.selectable_value(&mut self.material, i, *n);
                        }
                    });
            });
        });
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        if matches!(self.tab, Tab::Shell | Tab::Ico) {
            self.shell_preview(ui);
            return;
        }
        if self.tab == Tab::Cone {
            self.cone_preview(ui);
            return;
        }
        let stats_h = 64.0;
        let out_h = 44.0;
        let avail = ui.available_size();
        let prev_h = (avail.y - stats_h - out_h - 24.0).max(200.0);

        // ---- preview ----
        egui::Frame::new()
            .fill(PREVIEW)
            .corner_radius(CornerRadius::same(12))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x26, 0x35, 0x4f)))
            .show(ui, |ui| {
                let (resp, painter) = ui.allocate_painter(Vec2::new(avail.x, prev_h), egui::Sense::hover());
                let r = resp.rect;
                painter.text(r.left_top() + Vec2::new(14.0, 12.0), egui::Align2::LEFT_TOP, "TOP-DOWN PREVIEW", egui::FontId::proportional(12.0), Color32::from_rgb(0x8f, 0xa2, 0xc2));
                let mut lx = r.right() - 14.0;
                for (t, c) in [("true circle", TRUE_CIRCLE), ("wedges", WEDGE), ("filler", FILLER)] {
                    let g = painter.layout_no_wrap(t.into(), egui::FontId::proportional(11.5), Color32::from_rgb(0xa9, 0xb8, 0xd3));
                    lx -= g.size().x;
                    painter.galley(Pos2::new(lx, r.top() + 12.0), g.clone(), Color32::WHITE);
                    lx -= 15.0;
                    painter.rect_filled(egui::Rect::from_min_size(Pos2::new(lx, r.top() + 14.0), Vec2::splat(10.0)), 2.0, c);
                    lx -= 12.0;
                }
                match &self.built {
                    Some(Ok(b)) => {
                        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
                        for p in &b.pieces {
                            let (a, c, e, f) = p.bbox();
                            x0 = x0.min(a);
                            y0 = y0.min(c);
                            x1 = x1.max(e);
                            y1 = y1.max(f);
                        }
                        let area = r.shrink2(Vec2::new(24.0, 40.0));
                        let sc = (area.width() / (x1 - x0) as f32).min(area.height() / (y1 - y0) as f32);
                        let cx = (x0 + x1) as f32 / 2.0;
                        let cy = (y0 + y1) as f32 / 2.0;
                        let c = area.center() + Vec2::new(0.0, 10.0);
                        let to = |x: i32, y: i32| Pos2::new(c.x + (x as f32 - cx) * sc, c.y - (y as f32 - cy) * sc);
                        let stroke = if b.pieces.len() < 600 { Stroke::new(0.8_f32, OUTLINE) } else { Stroke::NONE };
                        for p in &b.pieces {
                            let pts: Vec<Pos2> = p.polygon().iter().map(|&(x, y)| to(x, y)).collect();
                            painter.add(egui::Shape::convex_polygon(pts, if p.is_wedge() { WEDGE } else { FILLER }, stroke));
                        }
                        // true circle (centre is (0,0) in layout coordinates)
                        let (ow, ol, hw, hl) = self.sizes();
                        let mut rings = vec![(ow, ol)];
                        if self.kind == Kind::Ring { rings.push((hw, hl)); }
                        for (rw, rl) in rings {
                            let pts: Vec<Pos2> = (0..=96).map(|k| {
                                let t = k as f32 / 96.0 * std::f32::consts::TAU;
                                let p = to(0, 0);
                                Pos2::new(p.x + rw as f32 * sc * t.cos(), p.y - rl as f32 * sc * t.sin())
                            }).collect();
                            painter.add(egui::Shape::line(pts, Stroke::new(1.0_f32, TRUE_CIRCLE.gamma_multiply(0.8))));
                        }
                    }
                    Some(Err(e)) => {
                        painter.text(r.center(), egui::Align2::CENTER_CENTER, e, egui::FontId::proportional(14.0), WARN);
                    }
                    None => {}
                }
            });
        ui.add_space(8.0);

        // ---- stats ----
        let (total, wedges) = match &self.built {
            Some(Ok(b)) => (b.bricks.len(), b.wedges),
            _ => (0, 0),
        };
        let stud = self.style.is_stud();
        let span = {
            let l = &self.settings().layout;
            let extra = if l.inverse && l.kind != Kind::Hollow { 2 * l.wall } else { 0 };
            let (ow, ol) = (l.width + extra, l.length + extra);
            let (w, h) = match l.shape {
                Shape::Full => (ow, ol),
                Shape::Half => (ow, ol / 2),
                Shape::Quarter => (ow / 2, ol / 2),
            };
            let st = |v: i32| if stud { fmt_num(v as f64) } else { fmt_num(v as f64 / 5.0) };
            let tall = if stud { fmt_num(self.height as f64 / 3.0) } else { fmt_num(self.height as f64 / 6.0) };
            (format!("{} × {}", st(w), st(h)), format!("studs · {tall} bricks tall"))
        };
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 3.0 * 8.0) / 4.0;
            for (big, small) in [
                (total.to_string(), "bricks in total".to_string()),
                (wedges.to_string(), "wedges".to_string()),
                ((total - wedges).to_string(), "filler bricks".to_string()),
                (span.0, span.1),
            ] {
                egui::Frame::new().fill(CARD).corner_radius(CornerRadius::same(10)).inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
                    ui.set_width(w - 24.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(bold(big, 20.0).color(Color32::WHITE));
                        ui.label(RichText::new(small).size(11.5).color(SOFT));
                    });
                });
            }
        });
        ui.add_space(8.0);

        // ---- output ----
        let ok = matches!(self.built, Some(Ok(_)));
        self.output_row(ui, ok);
    }

    fn output_row(&mut self, ui: &mut egui::Ui, ok: bool) {
        ui.horizontal(|ui| {
            let bw = 160.0;
            let r = ui.add_sized([ui.available_width() - 2.0 * bw - 16.0, 34.0], egui::TextEdit::singleline(&mut self.file_name).vertical_align(egui::Align::Center));
            if r.changed() {
                self.name_edited = true;
            }
            let save = egui::Button::new(bold("Save to Prefabs", 14.5).color(Color32::WHITE)).fill(GREEN).corner_radius(CornerRadius::same(9));
            if ui.add_enabled(ok, save).on_hover_text("Saves to %LOCALAPPDATA%\\Brickadia\\Saved\\Prefabs").clicked() {
                self.save(false);
            }
            let copy = egui::Button::new(bold("Copy to clipboard", 14.5).color(Color32::WHITE)).fill(ACCENT).corner_radius(CornerRadius::same(9));
            if ui.add_enabled(ok, copy).on_hover_text("Then paste in game with Ctrl+V").clicked() {
                self.copy();
            }
        });
        if self.name_edited {
            if ui.small_button("↺ Use automatic name").clicked() {
                self.name_edited = false;
                self.file_name = self.auto_name();
            }
        }
    }
}


// ======================================================================= Cone tab
impl App {
    fn cone_settings(&self) -> ConeSettings {
        let (bl, tl) = if self.c_oval { (self.c_bl, self.c_tl) } else { (self.c_bw, self.c_tw) };
        ConeSettings {
            shape: self.c_shape,
            ctype: self.c_type,
            levels: self.c_levels,
            bottom_w: self.c_bw,
            bottom_l: bl,
            top_w: self.c_tw,
            top_l: tl,
            height: self.c_h,
            block: self.c_block,
            band: self.c_band,
            fill: self.c_fill,
            upside_down: self.c_flip,
        }
    }

    fn cone_name(&self) -> String {
        let c = self.cone_settings();
        let shape = match c.shape {
            Shape::Full => "Full",
            Shape::Half => "Half",
            Shape::Quarter => "Quarter",
        };
        let ty = match c.ctype {
            ConeType::Solid => "Solid",
            ConeType::Hollow => "Hollow",
            ConeType::Inverse => "Hole",
            ConeType::Doughnut => "SlopedDoughnut",
        };
        let lv = match c.levels {
            Levels::Single => "SingleLevel",
            Levels::Multiple => "Levels",
        };
        let size = |w: i32, l: i32| if w == l { format!("{w}") } else { format!("{w}x{l}") };
        let fill = match (c.ctype, c.fill) {
            (ConeType::Hollow, _) => "",
            (_, ConeFill::LargeBlocks) => "_LargeBlocks",
            (_, ConeFill::Strips) => "_Strips",
        };
        let extra = match c.ctype {
            ConeType::Inverse => format!("_block{}", c.block),
            ConeType::Doughnut => format!("_band{}", c.band),
            _ => String::new(),
        };
        let base = if self.c_oval { "OvalCone" } else { "Cone" };
        format!(
            "{base}_{shape}_{ty}_{lv}_{}to{}M_{}M{extra}{fill}{}",
            size(c.bottom_w, c.bottom_l),
            size(c.top_w, c.top_l),
            c.height,
            if c.upside_down { "_UpsideDown" } else { "" }
        )
    }

    fn cone_bytes(&mut self) -> Option<(Vec<u8>, usize)> {
        let c = self.cone_settings();
        let r = match cone::build_cone(&c) {
            Ok(r) => r,
            Err(e) => {
                self.log(LogKind::Err, e);
                return None;
            }
        };
        let bad = cone::overlaps(&r.bricks);
        if bad > 0 {
            self.log(LogKind::Err, format!("Self-check failed ({bad} overlapping bricks). Nothing was saved. Please send this settings combination to Claude."));
            return None;
        }
        let w = cone_world(&r.bricks, (self.color[0], self.color[1], self.color[2]), MATERIALS[self.material].1, self.file_name.trim());
        match w.to_brz_vec() {
            Ok(b) => Some((b, r.bricks.len())),
            Err(e) => {
                self.log(LogKind::Err, format!("Could not create the prefab: {e}"));
                None
            }
        }
    }

    fn cone_controls(&mut self, ui: &mut egui::Ui) {
        let tmax = TYPE_MAX_MICRO;
        let smax = SLIDER_MAX_MICRO;
        card(ui, |ui| {
            ui.horizontal(|ui| {
                label(ui, "Shape");
                seg(ui, &mut self.c_shape, &[(Shape::Full, "● Full"), (Shape::Half, "◓ Half"), (Shape::Quarter, "◔ Quarter")]);
                ui.checkbox(&mut self.c_oval, bold("Oval", 13.5));
            });
            ui.horizontal(|ui| {
                label(ui, "Type");
                let before = self.c_type;
                seg(ui, &mut self.c_type, &[(ConeType::Solid, "Solid"), (ConeType::Hollow, "Hollow"), (ConeType::Inverse, "Inverse"), (ConeType::Doughnut, "Sloped doughnut")]);
                // a hole is wide at the top: swap the sizes when switching to or from Inverse
                if (before == ConeType::Inverse) != (self.c_type == ConeType::Inverse) {
                    std::mem::swap(&mut self.c_bw, &mut self.c_tw);
                    std::mem::swap(&mut self.c_bl, &mut self.c_tl);
                }
            });
            ui.horizontal(|ui| {
                label(ui, "Levels");
                seg(ui, &mut self.c_levels, &[(Levels::Single, "Single level"), (Levels::Multiple, "Multiple levels")]);
            });
            ui.horizontal(|ui| {
                label(ui, "Fill");
                if self.c_type == ConeType::Hollow {
                    ui.label(RichText::new("Hollow shapes have no filler").italics().color(SOFT));
                } else {
                    seg(ui, &mut self.c_fill, &[(ConeFill::LargeBlocks, "Large blocks"), (ConeFill::Strips, "Strips")]);
                }
            });
            ui.horizontal(|ui| {
                label(ui, "");
                ui.checkbox(&mut self.c_flip, bold("Upside down", 13.5));
            });
            ui.label(RichText::new("All cone pieces are micro bricks (the corner wedges only exist as micro bricks).").size(11.5).color(Color32::from_rgb(0x9f, 0xb6, 0xdc)));
        });
        ui.add_space(4.0);

        let even = if self.c_shape == Shape::Full { 1 } else { 2 };
        let note = |v: i32| format!("M ({})", studs(v as f64 / 5.0));
        card(ui, |ui| {
            let hole = self.c_type == ConeType::Inverse;
            let dough = self.c_type == ConeType::Doughnut;
            let (bname, tname) = if hole {
                ("Bottom opening", "Top opening")
            } else if dough {
                ("Slope bottom", "Band outside")
            } else {
                ("Bottom", "Top")
            };
            if !self.c_oval {
                let n = note(self.c_bw);
                size_row_s(ui, bname, &mut self.c_bw, 4, smax, tmax, n, true, even);
                let n = note(self.c_tw);
                size_row_s(ui, tname, &mut self.c_tw, 4, smax, tmax, n, true, even);
                self.c_bl = self.c_bw;
                self.c_tl = self.c_tw;
            } else {
                let n = note(self.c_bw);
                size_row_s(ui, &format!("{bname} width"), &mut self.c_bw, 4, smax, tmax, n, true, even);
                let n = note(self.c_bl);
                size_row_s(ui, &format!("{bname} length"), &mut self.c_bl, 4, smax, tmax, n, true, even);
                let n = note(self.c_tw);
                size_row_s(ui, &format!("{tname} width"), &mut self.c_tw, 4, smax, tmax, n, true, even);
                let n = note(self.c_tl);
                size_row_s(ui, &format!("{tname} length"), &mut self.c_tl, 4, smax, tmax, n, true, even);
            }
            let hn = format!("M ({} bricks / {} plates)", fmt_num(self.c_h as f64 / 6.0), fmt_num(self.c_h as f64 / 2.0));
            size_row(ui, "Height", &mut self.c_h, 1, 4000, 4000, hn, true);
            if hole {
                let n = note(self.c_block);
                size_row_s(ui, "Block size", &mut self.c_block, 6, smax, tmax, n, true, even);
            }
            if dough {
                let n = note(self.c_band);
                size_row(ui, "Band width", &mut self.c_band, 1, 1000, tmax / 2, n, true);
                let (hw, hl) = (self.c_tw - 2 * self.c_band, if self.c_oval { self.c_tl } else { self.c_tw } - 2 * self.c_band);
                let txt = if self.c_oval { format!("Hole: {hw} × {hl} M") } else { format!("Hole diameter: {hw} M") };
                ui.label(RichText::new(format!("                              {txt}")).size(12.5).strong().color(Color32::WHITE));
            }
            // slope angle
            let run = ((self.c_bw - self.c_tw).abs() as f64) / 2.0;
            if run > 0.0 {
                let ang = (self.c_h as f64 / run).atan().to_degrees();
                ui.label(RichText::new(format!("                              Slope: {} M across, {} M up → about {:.0}°", fmt_num(run), self.c_h, ang)).size(12.5).color(Color32::from_rgb(0xd6, 0xde, 0xf0)));
            }
            if let Some(Ok(r)) = &self.c_built {
                for w in &r.warnings {
                    ui.label(RichText::new(w).size(12.0).color(WARN));
                }
            }
        });
        ui.add_space(4.0);
    }

    fn cone_preview(&mut self, ui: &mut egui::Ui) {
        let stats_h = 64.0;
        let out_h = 44.0;
        let avail = ui.available_size();
        let prev_h = (avail.y - stats_h - out_h - 24.0).max(200.0);
        egui::Frame::new()
            .fill(PREVIEW)
            .corner_radius(CornerRadius::same(12))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x26, 0x35, 0x4f)))
            .show(ui, |ui| {
                let (resp, painter) = ui.allocate_painter(Vec2::new(avail.x, prev_h), egui::Sense::hover());
                let r = resp.rect;
                painter.text(r.left_top() + Vec2::new(14.0, 12.0), egui::Align2::LEFT_TOP, "TOP-DOWN PREVIEW (lighter = higher)", egui::FontId::proportional(12.0), Color32::from_rgb(0x8f, 0xa2, 0xc2));
                let mut lx = r.right() - 14.0;
                for (t, c) in [("slope wedges", WEDGE), ("filler", FILLER)] {
                    let g = painter.layout_no_wrap(t.into(), egui::FontId::proportional(11.5), Color32::from_rgb(0xa9, 0xb8, 0xd3));
                    lx -= g.size().x;
                    painter.galley(Pos2::new(lx, r.top() + 12.0), g.clone(), Color32::WHITE);
                    lx -= 15.0;
                    painter.rect_filled(egui::Rect::from_min_size(Pos2::new(lx, r.top() + 14.0), Vec2::splat(10.0)), 2.0, c);
                    lx -= 12.0;
                }
                match &self.c_built {
                    Some(Ok(res)) => {
                        let mut items: Vec<(i32, Vec<(f64, f64)>, bool)> =
                            res.bricks.iter().map(|b| (cone::top_z(b), cone::footprint_poly(b), b.asset == cone::A_TRI || b.asset == cone::A_OUT || b.dir != cone::Dir::ZP && b.dir != cone::Dir::ZN)).collect();
                        items.sort_by_key(|i| i.0);
                        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
                        let (mut zlo, mut zhi) = (i32::MAX, i32::MIN);
                        for (z, p, _) in &items {
                            zlo = zlo.min(*z);
                            zhi = zhi.max(*z);
                            for &(x, y) in p {
                                x0 = x0.min(x);
                                x1 = x1.max(x);
                                y0 = y0.min(y);
                                y1 = y1.max(y);
                            }
                        }
                        let area = r.shrink2(Vec2::new(24.0, 40.0));
                        let sc = (area.width() as f64 / (x1 - x0)).min(area.height() as f64 / (y1 - y0)) as f32;
                        let (cx, cy) = (((x0 + x1) / 2.0) as f32, ((y0 + y1) / 2.0) as f32);
                        let c = area.center() + Vec2::new(0.0, 10.0);
                        let to = |x: f64, y: f64| Pos2::new(c.x + (x as f32 - cx) * sc, c.y - (y as f32 - cy) * sc);
                        let stroke = if items.len() < 1500 { Stroke::new(0.6_f32, OUTLINE) } else { Stroke::NONE };
                        for (z, p, slope) in &items {
                            let t = if zhi > zlo { (*z - zlo) as f32 / (zhi - zlo) as f32 } else { 1.0 };
                            let base = if *slope { WEDGE } else { FILLER };
                            let col = base.gamma_multiply(0.45 + 0.55 * t);
                            let pts: Vec<Pos2> = p.iter().map(|&(x, y)| to(x, y)).collect();
                            painter.add(egui::Shape::convex_polygon(pts, col, stroke));
                        }
                    }
                    Some(Err(e)) => {
                        painter.text(r.center(), egui::Align2::CENTER_CENTER, e, egui::FontId::proportional(14.0), WARN);
                    }
                    None => {}
                }
            });
        ui.add_space(8.0);
        let (total, slope, levels) = match &self.c_built {
            Some(Ok(r)) => {
                let s = r.bricks.iter().filter(|b| b.asset == cone::A_TRI || b.asset == cone::A_OUT || matches!(b.dir, cone::Dir::XP | cone::Dir::XN | cone::Dir::YP | cone::Dir::YN)).count();
                let lv = if r.step > 1 { format!("{} levels · {} M steps", r.levels, r.step) } else { format!("{} level{}", r.levels, if r.levels == 1 { "" } else { "s" }) };
                (r.bricks.len(), s, lv)
            }
            _ => (0, 0, "-".into()),
        };
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 3.0 * 8.0) / 4.0;
            for (big, small) in [
                (total.to_string(), "bricks in total".to_string()),
                (slope.to_string(), "slope wedges".to_string()),
                ((total - slope).to_string(), "filler bricks".to_string()),
                (levels, format!("{} M tall", self.c_h)),
            ] {
                egui::Frame::new().fill(CARD).corner_radius(CornerRadius::same(10)).inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
                    ui.set_width(w - 24.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(bold(big, 20.0).color(Color32::WHITE));
                        ui.label(RichText::new(small).size(11.5).color(SOFT));
                    });
                });
            }
        });
        ui.add_space(8.0);
        let ok = matches!(self.c_built, Some(Ok(_)));
        self.output_row(ui, ok);
    }
}

// ======================================================================= Shell tab
/// Built-in outlines for the curve editor: (name, points in micro bricks, sharp corners).
fn shape_presets() -> Vec<(&'static str, Vec<(f64, f64)>, Vec<bool>)> {
    let bal = shell::balloon_points();
    let n = bal.len();
    vec![
        ("Your hot air balloon", bal, vec![false; n]),
        (
            "Zeppelin",
            vec![(0., 0.), (16., 6.), (28., 20.), (36., 45.), (40., 80.), (40., 130.), (37., 180.), (30., 225.), (20., 260.), (10., 285.), (0., 300.)],
            vec![false; 11],
        ),
        ("Egg", vec![(0., 0.), (30., 8.), (42., 30.), (44., 55.), (38., 80.), (24., 98.), (0., 108.)], vec![false; 7]),
        ("Vase", vec![(0., 0.), (30., 0.), (40., 30.), (28., 70.), (20., 90.), (26., 110.)], vec![false, true, false, false, false, false]),
        ("Onion dome", vec![(60., 0.), (70., 20.), (66., 45.), (45., 70.), (18., 90.), (6., 108.), (0., 125.)], vec![false; 7]),
    ]
}

impl App {
    fn shell_settings(&self) -> ShellSettings {
        let mut s = self.s_set.clone();
        s.sharp.resize(s.points.len(), false);
        s.material = MATERIALS[self.s_mat].1.to_string();
        s
    }

    fn shell_name(&self) -> String {
        let s = self.shell_settings();
        let shape = match s.curve {
            Curve::Cone => format!("Cone_{}to{}M_{}M", s.cone_bottom, s.cone_top, s.cone_height),
            Curve::Dish => format!("Dish_r{}M_d{}M", s.dish_radius, s.dish_depth),
            Curve::Sphere => match s.sphere_input {
                SphereInput::Angles => format!("Sphere_r{}M_{}to{}deg", s.sphere_radius, s.start_deg, s.end_deg),
                SphereInput::RadiusDepth => format!("SphereCap_r{}M_d{}M", s.cap_radius, s.cap_depth),
            },
            Curve::Custom => format!("Custom_{}pts", s.points.len()),
            Curve::Icosphere => {
                return format!(
                    "Icosphere_{}faces_{}studs_{}{}",
                    icosphere::face_count(s.ico_detail),
                    fmt_num(s.ico_diameter),
                    if s.ico_brick == IcoBrick::Side { "SideWedges" } else { "MicroWedges" },
                    if s.movement == Movement::Frozen { "_NoBearings" } else { "" }
                );
            }
        };
        format!(
            "Shell_{shape}_{}x{}_{}M{}{}",
            s.strips,
            s.rings,
            s.thickness,
            if s.bearings { "" } else { "_NoBearings" },
            if s.upside_down { "_UpsideDown" } else { "" }
        )
    }

    fn shell_bytes(&mut self) -> Option<(Vec<u8>, usize)> {
        let s = self.shell_settings();
        let name = self.file_name.trim().to_string();
        match shell::shell_bytes(&s, &name) {
            Ok((r, b)) => Some((b, r.brick_count())),
            Err(e) => {
                self.log(LogKind::Err, e);
                None
            }
        }
    }

    fn shell_controls(&mut self, ui: &mut egui::Ui) {
        let tmax = TYPE_MAX_MICRO;
        let smax = SLIDER_MAX_MICRO;
        let note = |m: i32| format!("M = {}", studs(m as f64 / 5.0));
        card(ui, |ui| {
            ui.horizontal(|ui| {
                label(ui, "Shape");
                if self.s_set.curve == Curve::Icosphere {
                    ui.label(bold("Icosphere", 15.0).color(Color32::WHITE));
                } else {
                    seg(ui, &mut self.s_set.curve, &[(Curve::Cone, "Cone"), (Curve::Dish, "Dish"), (Curve::Sphere, "Sphere"), (Curve::Custom, "Custom")]);
                }
            });
            let s = &mut self.s_set;
            match s.curve {
                Curve::Cone => {
                    { let nt = note(s.cone_bottom); size_row(ui, "Start radius", &mut s.cone_bottom, 0, smax, tmax, nt, true); }
                    { let nt = note(s.cone_top); size_row(ui, "End radius", &mut s.cone_top, 0, smax, tmax, nt, true); }
                    { let nt = note(s.cone_height); size_row(ui, "Height", &mut s.cone_height, 1, smax, tmax, nt, true); }
                    ui.label(RichText::new("The strips meet at the start end. A start radius of 0 makes a point.").size(12.0).color(SOFT));
                }
                Curve::Dish => {
                    { let nt = note(s.dish_radius); size_row(ui, "Radius", &mut s.dish_radius, 2, smax, tmax, nt, true); }
                    { let nt = note(s.dish_depth); size_row(ui, "Depth", &mut s.dish_depth, 1, smax, tmax, nt, true); }
                    let f = (s.dish_radius as f64).powi(2) / (4.0 * s.dish_depth.max(1) as f64);
                    ui.label(RichText::new(format!("Parabolic. Focal point {} M above the centre ({}).", fmt_num(f), studs(f / 5.0))).size(12.0).color(SOFT));
                }
                Curve::Sphere => {
                    ui.horizontal(|ui| {
                        label(ui, "Set by");
                        seg(ui, &mut s.sphere_input, &[(SphereInput::Angles, "Radius + angles"), (SphereInput::RadiusDepth, "Rim + depth")]);
                    });
                    match s.sphere_input {
                        SphereInput::Angles => {
                            { let nt = note(s.sphere_radius); size_row(ui, "Radius", &mut s.sphere_radius, 2, smax, tmax, nt, true); }
                            size_row(ui, "Start angle", &mut s.start_deg, 0, 179, 179, "°".into(), true);
                            size_row(ui, "End angle", &mut s.end_deg, 1, 180, 180, "°".into(), true);
                            ui.label(RichText::new("0° = the pole where the strips meet, 90° = halfway, 180° = the far pole.").size(12.0).color(SOFT));
                        }
                        SphereInput::RadiusDepth => {
                            { let nt = note(s.cap_radius); size_row(ui, "Rim radius", &mut s.cap_radius, 2, smax, tmax, nt, true); }
                            { let nt = note(s.cap_depth); size_row(ui, "Depth", &mut s.cap_depth, 1, smax, tmax, nt, true); }
                        }
                    }
                }
                Curve::Custom => {
                    self.curve_editor(ui);
                }
                Curve::Icosphere => {
                    self.ico_controls(ui);
                }
            }
            if self.s_set.curve != Curve::Icosphere {
                ui.checkbox(&mut self.s_set.upside_down, bold("Upside down (strips meet at the top)", 13.5));
            }
        });
        if self.s_set.curve == Curve::Icosphere {
            self.ico_colours(ui);
            return;
        }
        card(ui, |ui| {
            let s = &mut self.s_set;
            let mut strips = s.strips as i32;
            let mut rings = s.rings as i32;
            let nt = format!("{}° each", fmt_num(360.0 / strips.max(3) as f64));
            size_row(ui, "Strips", &mut strips, 3, 96, 720, nt, true);
            let nt = format!("{} panels", strips * rings);
            size_row(ui, "Rings", &mut rings, 1, 40, 400, nt, true);
            s.strips = strips.max(3) as u32;
            s.rings = rings.max(1) as u32;
            ui.horizontal(|ui| {
                label(ui, "Ring spacing");
                egui::ComboBox::from_id_salt("spacing")
                    .width(200.0)
                    .selected_text(match s.spacing {
                        Spacing::EqualLength => "Equal panel length",
                        Spacing::EqualHeight => "Equal height steps",
                        Spacing::EqualAngle => "Equal angle steps",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut s.spacing, Spacing::EqualLength, "Equal panel length");
                        ui.selectable_value(&mut s.spacing, Spacing::EqualHeight, "Equal height steps");
                        ui.selectable_value(&mut s.spacing, Spacing::EqualAngle, "Equal angle steps");
                    });
            });
            { let nt = note(s.thickness); size_row(ui, "Thickness", &mut s.thickness, 1, 20, 200, nt, true); }
            ui.horizontal(|ui| {
                label(ui, "Seams");
                egui::ComboBox::from_id_salt("seam")
                    .width(260.0)
                    .selected_text(match s.seam {
                        Seam::Outside => "Smooth outside (overlap inside)",
                        Seam::Inside => "Smooth inside (gaps outside)",
                        Seam::Middle => "Split evenly",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut s.seam, Seam::Outside, "Smooth outside (overlap inside)");
                        ui.selectable_value(&mut s.seam, Seam::Inside, "Smooth inside (gaps outside)");
                        ui.selectable_value(&mut s.seam, Seam::Middle, "Split evenly");
                    });
            });
        });
        card(ui, |ui| {
            let s = &mut self.s_set;
            ui.checkbox(&mut s.bearings, bold("Bearings (working hinges, like your dome)", 13.5));
            ui.add_enabled_ui(s.bearings, |ui| {
                ui.checkbox(&mut s.hide_bearings, "Hide bearings (invisible, no collision)");
                ui.checkbox(&mut s.column_inside, "Centre column inside the shape");
                if s.bearings && s.thickness == 1 {
                    ui.label(RichText::new("⚠ 1-micro panels can't hide their bearings. Choose which face they sit on:").size(12.5).color(WARN));
                    ui.horizontal(|ui| {
                        label(ui, "Bearing face");
                        seg(ui, &mut s.bearing_side, &[(BearingSide::Inside, "Inside"), (BearingSide::Outside, "Outside")]);
                    });
                }
            });
            if !s.bearings {
                ui.label(RichText::new("⚠ Without bearings the panels paste frozen and won't hold their angle if unfrozen.").size(12.0).color(WARN));
            }
        });
        card(ui, |ui| {
            let s = &mut self.s_set;
            ui.horizontal(|ui| {
                label(ui, "Colours");
                seg(ui, &mut s.colour_mode, &[(ColourMode::One, "One"), (ColourMode::PerRing, "Per ring"), (ColourMode::PerStrip, "Per strip"), (ColourMode::Checker, "Checker")]);
            });
            let shown = if s.colour_mode == ColourMode::One { 1 } else { s.colours.len() };
            ui.horizontal_wrapped(|ui| {
                label(ui, "");
                for c in s.colours.iter_mut().take(shown) {
                    egui::color_picker::color_edit_button_srgb(ui, c);
                }
                if s.colour_mode != ColourMode::One {
                    if s.colours.len() < 8 && ui.small_button("+").clicked() {
                        let last = *s.colours.last().unwrap();
                        s.colours.push(last);
                    }
                    if s.colours.len() > 1 && ui.small_button("−").clicked() {
                        s.colours.pop();
                    }
                }
            });
            ui.horizontal(|ui| {
                label(ui, "Material");
                egui::ComboBox::from_id_salt("s_material")
                    .width(150.0)
                    .selected_text(MATERIALS[self.s_mat].0)
                    .show_ui(ui, |ui| {
                        for (i, (n, _)) in MATERIALS.iter().enumerate() {
                            ui.selectable_value(&mut self.s_mat, i, *n);
                        }
                    });
            });
        });
    }

    fn shell_preview(&mut self, ui: &mut egui::Ui) {
        let stats_h = 64.0;
        let out_h = 44.0;
        let avail = ui.available_size();
        let warn_h = match &self.s_built {
            Some(Ok(r)) if !r.warnings.is_empty() => 20.0 * r.warnings.len() as f32,
            _ => 0.0,
        };
        let prev_h = (avail.y - stats_h - out_h - 24.0 - warn_h).max(200.0);
        let colours = self.s_set.colours.clone();
        egui::Frame::new()
            .fill(PREVIEW)
            .corner_radius(CornerRadius::same(12))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x26, 0x35, 0x4f)))
            .show(ui, |ui| {
                let (resp, painter) = ui.allocate_painter(Vec2::new(avail.x, prev_h), egui::Sense::drag());
                if resp.dragged() {
                    let d = resp.drag_delta();
                    self.s_view.0 -= d.x * 0.4;
                    self.s_view.1 = (self.s_view.1 + d.y * 0.4).clamp(-89.0, 89.0);
                }
                let r = resp.rect;
                painter.text(r.left_top() + Vec2::new(14.0, 12.0), egui::Align2::LEFT_TOP, "3D PREVIEW (drag to turn)", egui::FontId::proportional(12.0), Color32::from_rgb(0x8f, 0xa2, 0xc2));
                match &self.s_built {
                    Some(Ok(res)) => {
                        let (yaw, pitch) = (self.s_view.0.to_radians() as f64, self.s_view.1.to_radians() as f64);
                        let view = |p: [f64; 3]| -> (f64, f64, f64) {
                            let (sy, cy) = yaw.sin_cos();
                            let (x, y, z) = (p[0] * cy - p[1] * sy, p[0] * sy + p[1] * cy, p[2]);
                            let (sp, cp) = pitch.sin_cos();
                            // screen x, screen up, depth (bigger = nearer)
                            (x, z * cp - y * sp, -(y * cp + z * sp))
                        };
                        let mut quads: Vec<(f64, Vec<(f64, f64)>, Color32)> = vec![];
                        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
                        for p in &res.panels {
                            let m = &p.rot;
                            let corner = |lx: f64, ly: f64| {
                                let w = shell::mv(m, [lx, ly, 0.0]);
                                view([p.loc[0] + w[0], p.loc[1] + w[1], p.loc[2] + w[2]])
                            };
                            if p.shape.is_empty() {
                                continue; // the hidden hub
                            }
                            let c: Vec<(f64, f64, f64)> = p.shape.iter().map(|&(x, y)| corner(x, y)).collect();
                            let depth = c.iter().map(|q| q.2).sum::<f64>() / c.len() as f64;
                            let n = [m[0][2], m[1][2], m[2][2]];
                            let nv = view(n);
                            let light = 0.35 + 0.65 * (nv.2.abs() * 0.7 + nv.1.abs() * 0.3).min(1.0);
                            let k = colours[p.colour % colours.len()];
                            let col = Color32::from_rgb((k[0] as f64 * light) as u8, (k[1] as f64 * light) as u8, (k[2] as f64 * light) as u8);
                            for q in &c {
                                x0 = x0.min(q.0);
                                x1 = x1.max(q.0);
                                y0 = y0.min(q.1);
                                y1 = y1.max(q.1);
                            }
                            quads.push((depth, c.iter().map(|q| (q.0, q.1)).collect(), col));
                        }
                        quads.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                        let area = r.shrink2(Vec2::new(24.0, 40.0));
                        let sc = (area.width() as f64 / (x1 - x0).max(1.0)).min(area.height() as f64 / (y1 - y0).max(1.0)) as f32;
                        let (cx, cy) = (((x0 + x1) / 2.0) as f32, ((y0 + y1) / 2.0) as f32);
                        let c = area.center() + Vec2::new(0.0, 10.0);
                        let to = |x: f64, y: f64| Pos2::new(c.x + (x as f32 - cx) * sc, c.y - (y as f32 - cy) * sc);
                        let stroke = Stroke::new(0.6_f32, OUTLINE);
                        for (_, q, col) in &quads {
                            let mut pts: Vec<Pos2> = vec![];
                            for &(x, y) in q.iter() {
                                let pt = to(x, y);
                                // drop repeated corners (pointed panels), they upset egui's edge smoothing
                                if pts.iter().all(|o: &Pos2| o.distance(pt) > 0.3) {
                                    pts.push(pt);
                                }
                            }
                            if pts.len() < 3 {
                                continue;
                            }
                            let k = pts.len();
                            // same winding for every panel (egui's edge smoothing needs it)
                            let area2: f32 = (0..k).map(|i| { let (a, b) = (pts[i], pts[(i + 1) % k]); a.x * b.y - b.x * a.y }).sum();
                            let longest = (0..k).map(|i| pts[i].distance(pts[(i + 1) % k])).fold(0.0f32, f32::max);
                            if area2.abs() < 0.5 || area2.abs() / (longest * longest).max(1e-3) < 0.02 {
                                // seen edge-on: draw as lines (egui's smoothing spikes on slivers)
                                for i in 0..k {
                                    painter.line_segment([pts[i], pts[(i + 1) % k]], Stroke::new(1.0_f32, *col));
                                }
                                continue;
                            }
                            if area2 < 0.0 {
                                pts.reverse();
                            }
                            painter.add(egui::Shape::convex_polygon(pts, *col, stroke));
                        }
                    }
                    Some(Err(e)) => {
                        painter.text(r.center(), egui::Align2::CENTER_CENTER, e, egui::FontId::proportional(14.0), WARN);
                    }
                    None => {}
                }
            });
        if let Some(Ok(res)) = &self.s_built {
            for w in &res.warnings {
                ui.label(RichText::new(format!("⚠ {w}")).size(12.5).color(WARN));
            }
        }
        ui.add_space(8.0);
        let (ents, bricks, bearings, size) = match &self.s_built {
            Some(Ok(r)) => {
                if self.s_set.curve == Curve::Icosphere {
                    let faces = r.panels.iter().filter(|p| !p.shape.is_empty()).count();
                    (faces.to_string(), r.brick_count().to_string(), r.bearing_count().to_string(), format!("{}|wide · gaps ≤ {:.2} stud", fmt_num(self.s_set.ico_diameter), r.gap / 10.0))
                } else {
                    let rmax = r.outline.iter().map(|p| p.0).fold(0.0, f64::max);
                    let (zlo, zhi) = r.outline.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
                    (r.panels.len().to_string(), r.brick_count().to_string(), r.bearing_count().to_string(), format!("{} × {}|studs wide × tall", fmt_num((rmax * 2.0 / 10.0 * 10.0).round() / 10.0), fmt_num(((zhi - zlo) / 10.0 * 10.0).round() / 10.0)))
                }
            }
            _ => ("-".into(), "-".into(), "-".into(), "-".into()),
        };
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 3.0 * 8.0) / 4.0;
            for (big, small) in [
                (ents, "panels (entities)".to_string()),
                (bricks, "bricks in total".to_string()),
                (bearings, "bearings".to_string()),
                (size.split('|').next().unwrap_or("-").to_string(), size.split('|').nth(1).unwrap_or("studs").to_string()),
            ] {
                egui::Frame::new().fill(CARD).corner_radius(CornerRadius::same(10)).inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
                    ui.set_width(w - 24.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(bold(big, 20.0).color(Color32::WHITE));
                        ui.label(RichText::new(small).size(11.5).color(SOFT));
                    });
                });
            }
        });
        ui.add_space(8.0);
        let ok = matches!(self.s_built, Some(Ok(_)));
        self.output_row(ui, ok);
    }
}

impl App {
    // ------------------------------------------------------------ icosphere
    fn ico_controls(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.s_set;
        ui.horizontal(|ui| {
            label(ui, "Faces");
            seg(ui, &mut s.ico_detail, &[(1, "20"), (2, "80"), (3, "180"), (4, "320"), (5, "500"), (6, "720")]);
        });
        ui.horizontal(|ui| {
            label(ui, "Bricks");
            seg(ui, &mut s.ico_brick, &[(IcoBrick::Micro, "Micro wedges"), (IcoBrick::Side, "Side Wedges")]);
        });
        ui.horizontal(|ui| {
            label(ui, "Diameter");
            ui.add(egui::DragValue::new(&mut s.ico_diameter).range(2.0..=4000.0).speed(0.1).fixed_decimals(2).suffix(" studs"));
        });
        // best sizes for this detail and brick (cached)
        let key = (s.ico_detail, s.ico_brick);
        if self.s_best.as_ref().map(|b| b.0) != Some(key) {
            self.s_best = Some((key, icosphere::best_sizes(key.0, key.1, 10.0, 1500.0, 8)));
        }
        if let Some((_, list)) = &self.s_best {
            ui.label(RichText::new("Best sizes (click to use). Gap = worst gap at a corner; nothing ever sticks past an edge:").size(12.0).color(SOFT));
            ui.horizontal_wrapped(|ui| {
                for &(d, g) in list {
                    let txt = format!("{} studs · gap {:.2}", fmt_num(d), g);
                    if ui.small_button(txt).clicked() {
                        self.s_set.ico_diameter = d;
                    }
                }
            });
        }
        let s = &mut self.s_set;
        if s.ico_brick == IcoBrick::Side {
            let mut plates = s.ico_plates;
            size_row(ui, "Thickness", &mut plates, 1, 20, 20, "plates".into(), true);
            s.ico_plates = plates;
        } else {
            let nt = format!("M = {}", studs(s.thickness as f64 / 5.0));
            size_row(ui, "Thickness", &mut s.thickness, 1, 20, 40, nt, true);
        }
        // same as the other tools: frozen, or held by bearings
        let mut bearings = s.movement != Movement::Frozen;
        ui.checkbox(&mut bearings, bold("Bearings (every face held from a hub on one ground bearing)", 13.5));
        s.movement = if bearings { Movement::Spin } else { Movement::Frozen };
        ui.add_enabled_ui(bearings, |ui| {
            ui.checkbox(&mut s.hide_bearings, "Hide the face bearings (invisible, no collision)");
        });
        if bearings {
            ui.label(RichText::new("Turn the ground bearing to turn the whole ball.").size(12.0).color(SOFT));
        } else {
            ui.label(RichText::new("⚠ Without bearings the faces paste frozen and won't hold together if unfrozen.").size(12.0).color(WARN));
        }
    }

    fn ico_colours(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            let s = &mut self.s_set;
            ui.checkbox(&mut s.ico_two_tone, bold("Two-tone: colour the faces round the 12 five-way corners", 13.5));
            if s.colours.len() < 2 {
                s.colours.push([224, 85, 75]);
            }
            ui.horizontal(|ui| {
                label(ui, "Colours");
                egui::color_picker::color_edit_button_srgb(ui, &mut s.colours[0]);
                if s.ico_two_tone {
                    egui::color_picker::color_edit_button_srgb(ui, &mut s.colours[1]);
                }
            });
            ui.horizontal(|ui| {
                label(ui, "Material");
                egui::ComboBox::from_id_salt("i_material")
                    .width(150.0)
                    .selected_text(MATERIALS[self.s_mat].0)
                    .show_ui(ui, |ui| {
                        for (i, (n, _)) in MATERIALS.iter().enumerate() {
                            ui.selectable_value(&mut self.s_mat, i, *n);
                        }
                    });
            });
        });
    }

    // ------------------------------------------------------------ curve editor
    /// Side view of the custom outline: drag points, double-click the line to add one,
    /// right-click a point for sharp corner / delete. The left half mirrors the right.
    fn curve_editor(&mut self, ui: &mut egui::Ui) {
        // presets
        ui.horizontal(|ui| {
            label(ui, "Start from");
            egui::ComboBox::from_id_salt("presets").width(170.0).selected_text("choose a shape…").show_ui(ui, |ui| {
                for (name, pts, sharp) in shape_presets() {
                    if ui.selectable_label(false, name).clicked() {
                        self.s_set.points = pts;
                        self.s_set.sharp = sharp;
                        self.s_sel = None;
                    }
                }
            });
        });
        let n = self.s_set.points.len();
        self.s_set.sharp.resize(n, false);
        let w = ui.available_width().max(200.0);
        let (resp, painter) = ui.allocate_painter(Vec2::new(w, w * 0.8), egui::Sense::click_and_drag());
        let rect = resp.rect;
        painter.rect_filled(rect, CornerRadius::same(8), PREVIEW);
        // view box (frozen while dragging so the view doesn't jump)
        let pts = self.s_set.points.clone();
        let rmax = pts.iter().map(|p| p.0).fold(10.0, f64::max);
        let (zlo, zhi) = pts.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
        let span = (zhi - zlo).max(rmax * 2.0).max(10.0);
        let live = (rmax * 1.2, zlo - span * 0.1, zhi + span * 0.1, 0.0);
        let bx = if self.s_drag.is_some() { self.s_ebox.unwrap_or(live) } else { live };
        self.s_ebox = Some(bx);
        let (half_w, z0, z1) = (bx.0, bx.1, bx.2);
        let inner = rect.shrink(12.0);
        let scale = (inner.width() as f64 / (2.0 * half_w)).min(inner.height() as f64 / (z1 - z0));
        let cx = inner.center().x as f64;
        let zmid = (z0 + z1) / 2.0;
        let cy = inner.center().y as f64;
        let to = |r: f64, z: f64| Pos2::new((cx + r * scale) as f32, (cy - (z - zmid) * scale) as f32);
        let from = |p: Pos2| -> (f64, f64) { ((p.x as f64 - cx) / scale, zmid - (p.y as f64 - cy) / scale) };
        // grid: every 5 studs (25 M), or 1 stud when zoomed in
        let grid = if 5.0 * scale > 14.0 { 5.0 } else if 25.0 * scale > 10.0 { 25.0 } else { 125.0 };
        let gcol = Color32::from_rgb(0x1f, 0x2c, 0x45);
        let mut g = (-(half_w / grid).ceil()) * grid;
        while g <= half_w {
            painter.line_segment([to(g, z0), to(g, z1)], Stroke::new(1.0_f32, gcol));
            g += grid;
        }
        let mut g = (z0 / grid).ceil() * grid;
        while g <= z1 {
            painter.line_segment([to(-half_w, g), to(half_w, g)], Stroke::new(1.0_f32, gcol));
            g += grid;
        }
        painter.line_segment([to(0.0, z0), to(0.0, z1)], Stroke::new(1.0_f32, Color32::from_rgb(0x55, 0x66, 0x88)));
        // the smooth curve, mirrored, with ring marks from the last build
        let mut tmp = self.s_set.clone();
        tmp.curve = Curve::Custom;
        tmp.upside_down = false;
        if let Ok(line) = shell::outline(&tmp) {
            for side in [-1.0, 1.0] {
                let poly: Vec<Pos2> = line.iter().map(|&(r, z)| to(side * r / 2.0, z / 2.0)).collect();
                painter.add(egui::Shape::line(poly, Stroke::new(2.0_f32, TRUE_CIRCLE)));
            }
            let (rp, _) = shell::ring_points(&tmp, &line);
            for &(r, z) in &rp {
                for side in [-1.0, 1.0] {
                    painter.circle_filled(to(side * r / 2.0, z / 2.0), 2.5, Color32::from_rgb(0x8f, 0xa2, 0xc2));
                }
            }
        }
        // points: real ones on the right, ghosts on the left
        for (i, &(r, z)) in pts.iter().enumerate() {
            let sel = self.s_sel == Some(i);
            let sharp = self.s_set.sharp[i];
            let col = if sel { WARN } else { Color32::WHITE };
            for (side, real) in [(1.0, true), (-1.0, false)] {
                let c = to(side * r, z);
                let col = if real { col } else { col.gamma_multiply(0.35) };
                if sharp {
                    painter.rect_filled(egui::Rect::from_center_size(c, Vec2::splat(9.0)), CornerRadius::ZERO, col);
                } else {
                    painter.circle_filled(c, 5.0, col);
                }
            }
        }
        // interaction
        let shift = ui.input(|i| i.modifiers.shift);
        let snap_on = self.s_snap != shift;
        let step = if self.s_snap_stud { 5.0 } else { 1.0 };
        let hit = |p: Pos2| -> Option<usize> {
            let (r, z) = from(p);
            let mut best: Option<(usize, f64)> = None;
            for (i, &(pr, pz)) in pts.iter().enumerate() {
                let d = (((r.abs() - pr) * scale).powi(2) + ((z - pz) * scale).powi(2)).sqrt();
                if d < 9.0 && best.map_or(true, |b| d < b.1) {
                    best = Some((i, d));
                }
            }
            best.map(|b| b.0)
        };
        if resp.drag_started() {
            // test where the button went down, not where the drag was noticed
            if let Some(p) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos()) {
                self.s_drag = hit(p);
                if self.s_drag.is_some() {
                    self.s_sel = self.s_drag;
                }
            }
        }
        if resp.dragged() {
            if let (Some(i), Some(p)) = (self.s_drag, resp.interact_pointer_pos()) {
                let (mut r, mut z) = from(p);
                r = r.abs();
                if snap_on {
                    r = (r / step).round() * step;
                    z = (z / step).round() * step;
                }
                self.s_set.points[i] = (r.max(0.0), z);
            }
        }
        if resp.drag_stopped() {
            self.s_drag = None;
        }
        if resp.clicked() {
            self.s_sel = resp.interact_pointer_pos().and_then(hit);
        }
        if resp.double_clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                if hit(p).is_none() {
                    // insert between the two points nearest along the outline
                    let (mut r, mut z) = from(p);
                    r = r.abs();
                    if snap_on {
                        r = (r / step).round() * step;
                        z = (z / step).round() * step;
                    }
                    let seg_d = |a: (f64, f64), b: (f64, f64)| {
                        let (dx, dz) = (b.0 - a.0, b.1 - a.1);
                        let l2 = (dx * dx + dz * dz).max(1e-9);
                        let t = (((r - a.0) * dx + (z - a.1) * dz) / l2).clamp(0.0, 1.0);
                        ((a.0 + t * dx - r).powi(2) + (a.1 + t * dz - z).powi(2)).sqrt()
                    };
                    let k = (0..pts.len() - 1).min_by(|&a, &b| seg_d(pts[a], pts[a + 1]).partial_cmp(&seg_d(pts[b], pts[b + 1])).unwrap()).unwrap_or(0);
                    self.s_set.points.insert(k + 1, (r, z));
                    self.s_set.sharp.insert(k + 1, false);
                    self.s_sel = Some(k + 1);
                }
            }
        }
        if resp.secondary_clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                self.s_sel = hit(p);
            }
        }
        let sel_for_menu = self.s_sel;
        resp.context_menu(|ui| {
            if let Some(i) = sel_for_menu {
                let mut sharp = self.s_set.sharp[i];
                if ui.checkbox(&mut sharp, "Sharp corner").changed() {
                    self.s_set.sharp[i] = sharp;
                    ui.close();
                }
                if ui.add_enabled(self.s_set.points.len() > 2, egui::Button::new("Delete point")).clicked() {
                    self.s_set.points.remove(i);
                    self.s_set.sharp.remove(i);
                    self.s_sel = None;
                    ui.close();
                }
            } else {
                ui.label("Right-click a point");
            }
        });
        if let Some(i) = self.s_sel {
            if i < self.s_set.points.len() && ui.input(|k| k.key_pressed(egui::Key::Delete)) && self.s_set.points.len() > 2 {
                self.s_set.points.remove(i);
                self.s_set.sharp.remove(i);
                self.s_sel = None;
            }
        }
        ui.label(RichText::new("Drag points · double-click the line to add · right-click a point for sharp corner / delete").size(11.5).color(SOFT));
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.s_snap, "Snap");
            seg(ui, &mut self.s_snap_stud, &[(false, "1 micro"), (true, "1 stud")]);
            ui.label(RichText::new("(hold Shift to flip)").size(11.5).color(SOFT));
        });
        // selected point: exact numbers
        if let Some(i) = self.s_sel.filter(|&i| i < self.s_set.points.len()) {
            ui.horizontal(|ui| {
                label(ui, &format!("Point {}", i + 1));
                let (r, z) = &mut self.s_set.points[i];
                ui.add(egui::DragValue::new(r).range(0.0..=20000.0).speed(0.5).prefix("radius ").suffix(" M"));
                ui.add(egui::DragValue::new(z).speed(0.5).prefix("height ").suffix(" M"));
                ui.checkbox(&mut self.s_set.sharp[i], "sharp");
            });
        }
        // scaler: whole-shape width / height in studs
        let pts = self.s_set.points.clone();
        let rmax = pts.iter().map(|p| p.0).fold(0.0, f64::max);
        let (zlo, zhi) = pts.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
        let (mut wid, mut hei) = (rmax * 2.0 / 5.0, (zhi - zlo) / 5.0);
        let (w0, h0) = (wid, hei);
        ui.horizontal(|ui| {
            label(ui, "Size");
            ui.add(egui::DragValue::new(&mut wid).range(0.4..=8000.0).speed(0.2).fixed_decimals(1).prefix("wide ").suffix(" st"));
            ui.add(egui::DragValue::new(&mut hei).range(0.4..=8000.0).speed(0.2).fixed_decimals(1).prefix("tall ").suffix(" st"));
            ui.checkbox(&mut self.s_lock, "keep shape");
        });
        ui.horizontal(|ui| {
            label(ui, "");
            for (txt, f) in [("×½", 0.5), ("×0.9", 0.9), ("×1.1", 1.1), ("×2", 2.0)] {
                if ui.small_button(txt).clicked() {
                    wid = w0 * f;
                    hei = h0 * f;
                }
            }
        });
        let (mut fx, mut fz) = (if w0 > 1e-9 { wid / w0 } else { 1.0 }, if h0 > 1e-9 { hei / h0 } else { 1.0 });
        if self.s_lock {
            if (fx - 1.0).abs() > 1e-9 {
                fz = fx;
            } else if (fz - 1.0).abs() > 1e-9 {
                fx = fz;
            }
        }
        if (fx - 1.0).abs() > 1e-9 || (fz - 1.0).abs() > 1e-9 {
            for p in &mut self.s_set.points {
                p.0 = (p.0 * fx * 100.0).round() / 100.0;
                p.1 = zlo + ((p.1 - zlo) * fz * 100.0).round() / 100.0;
            }
        }
    }
}

fn main() -> eframe::Result {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Circle Generator")
            .with_inner_size([1180.0, 800.0])
            .with_min_inner_size([980.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native("Circle Generator", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
