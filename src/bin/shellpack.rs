//! Writes the Shell tab test pack (v0.4.0) using exactly the engine the app uses.
use circlegen::shell::*;
fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or("shellpack".into()));
    std::fs::create_dir_all(&out).unwrap();
    let base = ShellSettings::default();
    let red = [224u8, 85, 75];
    let white = [255u8, 255, 255];
    let blue = [60u8, 110, 200];
    let dome = ShellSettings { material: "BMC_Metallic".into(), ..base.clone() };
    let sphere = ShellSettings { curve: Curve::Sphere, sphere_radius: 100, start_deg: 0, end_deg: 180, strips: 16, rings: 12, thickness: 2, ..base.clone() };
    let balloon = ShellSettings { curve: Curve::Custom, points: balloon_points(), strips: 12, rings: 11, thickness: 2, colour_mode: ColourMode::PerStrip, colours: vec![red, white], ..base.clone() };
    let list: Vec<(&str, ShellSettings)> = vec![
        ("S01_Dish_YourDome_r320_d105_24x8_3M", dome.clone()),
        ("S02_Dish_YourDome_NoBearings", ShellSettings { bearings: false, ..dome.clone() }),
        ("S03_Sphere_r100_16x12_2M_Checker", ShellSettings { colour_mode: ColourMode::Checker, colours: vec![white, blue], ..sphere.clone() }),
        ("S04_Dome_UpsideDown_r120_0to90_24x8_PerRing", ShellSettings { sphere_radius: 120, end_deg: 90, strips: 24, rings: 8, upside_down: true, colour_mode: ColourMode::PerRing, colours: vec![white, red], ..sphere.clone() }),
        ("S05_Balloon_Yours_12x11_2M", balloon.clone()),
        ("S06_Balloon_Smoother_24x16_2M", ShellSettings { strips: 24, rings: 16, ..balloon.clone() }),
        ("S07_Dish_1M_Thin_BearingsInside_Visible", ShellSettings { dish_radius: 150, dish_depth: 50, strips: 16, rings: 6, thickness: 1, hide_bearings: false, ..base.clone() }),
        ("S08_Cone_Lampshade_40to120_h60_16x4_EqualHeight", ShellSettings { curve: Curve::Cone, strips: 16, rings: 4, thickness: 2, spacing: Spacing::EqualHeight, ..base.clone() }),
        ("S09_Dish_EqualAngle_SmoothInside", ShellSettings { dish_radius: 200, dish_depth: 80, strips: 20, rings: 6, spacing: Spacing::EqualAngle, seam: Seam::Inside, ..base.clone() }),
        ("S10_SphereCap_Rim150_Depth60_ColumnInside_BearingsVisible", ShellSettings { curve: Curve::Sphere, sphere_input: SphereInput::RadiusDepth, cap_radius: 150, cap_depth: 60, strips: 18, rings: 5, column_inside: true, hide_bearings: false, ..base.clone() }),
    ];
    for (name, s) in list {
        match shell_bytes(&s, name) {
            Ok((r, bytes)) => {
                std::fs::write(out.join(format!("{name}.brz")), bytes).unwrap();
                // panel corners for the contact sheet
                let mut q = String::new();
                for p in &r.panels {
                    let h = p.length / 2.0;
                    let k = s.colours[p.colour % s.colours.len()];
                    for (lx, ly) in [(-h, -p.w_in / 2.0), (h, -p.w_out / 2.0), (h, p.w_out / 2.0), (-h, p.w_in / 2.0)] {
                        let w = mv(&p.rot, [lx, ly, 0.0]);
                        q += &format!("{:.2} {:.2} {:.2} ", p.loc[0] + w[0], p.loc[1] + w[1], p.loc[2] + w[2]);
                    }
                    q += &format!("{} {} {}\n", k[0], k[1], k[2]);
                }
                std::fs::write(out.join(format!("{name}.quads")), q).unwrap();
                println!("{name}: {} entities, {} bricks, {} bearings, check {:?}, warnings {:?}", r.panels.len(), r.brick_count(), r.bearing_count(), check_shell(&r), r.warnings);
            }
            Err(e) => println!("{name}: ERR {e}"),
        }
    }
}
