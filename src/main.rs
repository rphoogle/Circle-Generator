//! Circle Generator: the window. All the shape-making lives in the library
//! (src/lib.rs + src/geometry.rs); this file only draws controls and the
//! preview, and saves / copies the result.
#![cfg_attr(windows, windows_subsystem = "windows")]

use circlegen::{build, check, to_world, BrickStyle, Fill, Kind, Layout, Settings, Shape};
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

struct App {
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
        self.log(LogKind::Ok, format!("Saved {n} bricks to {} (checked: no gaps, no overlaps).", path.display()));
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
            Ok(()) => self.log(LogKind::Ok, format!("Copied {n} bricks. In game, paste with Ctrl+V (checked: no gaps, no overlaps).")),
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

/// Slider + type-in box. Slider stops at `slider_max`; the box goes to `type_max`.
fn size_row(ui: &mut egui::Ui, name: &str, v: &mut i32, min: i32, slider_max: i32, type_max: i32, note: String, enabled: bool) -> bool {
    let before = *v;
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            label(ui, name);
            let mut sv = (*v).min(slider_max);
            if ui.add(egui::Slider::new(&mut sv, min..=slider_max).logarithmic(slider_max > 100).show_value(false)).changed() {
                *v = sv;
            }
            ui.add_sized([62.0, 22.0], egui::DragValue::new(v).range(min..=type_max).speed(0.2));
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
                        ui.label(bold("Smooth circles from Micro Wedges or Side Wedges, saved as Brickadia prefabs.", 13.0).color(SOFT));
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
                size_row(ui, if which == "Hole " { "Hole diameter" } else { "Diameter" }, &mut self.width, dlo, smax, tmax, n, true);
                // radius, linked
                ui.horizontal(|ui| {
                    label(ui, if which == "Hole " { "Hole radius" } else { "Radius" });
                    let mut r = self.width as f64 / 2.0;
                    let mut sr = r.min(smax as f64 / 2.0);
                    if ui.add(egui::Slider::new(&mut sr, (dmin as f64 / 2.0)..=(smax as f64 / 2.0)).logarithmic(true).show_value(false).step_by(0.5)).changed() {
                        self.width = (sr * 2.0).round() as i32;
                    }
                    if ui.add_sized([62.0, 22.0], egui::DragValue::new(&mut r).range(1.0..=(tmax as f64 / 2.0)).speed(0.1).max_decimals(1).custom_formatter(|v, _| fmt_num(v))).changed() {
                        self.width = (r * 2.0).round() as i32;
                    }
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
