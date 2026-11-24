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
    let ico = ShellSettings { curve: Curve::Icosphere, ..base.clone() };
    let mut list = list;
    list.extend(vec![
        ("V01_Icosphere_80faces_71.2studs_SideWedges_Frozen", ShellSettings { ico_detail: 2, ico_diameter: 71.2, ico_brick: IcoBrick::Side, ..ico.clone() }),
        ("V02_Icosphere_180faces_317.2studs_SideWedges_Spin", ShellSettings { ico_detail: 3, ico_diameter: 317.2, ico_brick: IcoBrick::Side, movement: Movement::Spin, ico_two_tone: true, colours: vec![white, red], ..ico.clone() }),
        ("V03_Icosphere_720faces_1244.98studs_Micro_Free", ShellSettings { ico_detail: 6, ico_diameter: 1244.98, ico_brick: IcoBrick::Micro, thickness: 2, movement: Movement::Free, colours: vec![[60, 160, 70]], ..ico.clone() }),
        ("V04_Custom_Vase_SharpBase_16x8", ShellSettings { curve: Curve::Custom, points: vec![(0.0, 0.0), (30.0, 0.0), (40.0, 30.0), (28.0, 70.0), (20.0, 90.0), (26.0, 110.0)], sharp: vec![false, true, false, false, false, false], strips: 16, rings: 8, thickness: 2, ..base.clone() }),
        ("V05_Custom_Zeppelin_16x20", ShellSettings { curve: Curve::Custom, points: vec![(0., 0.), (16., 6.), (28., 20.), (36., 45.), (40., 80.), (40., 130.), (37., 180.), (30., 225.), (20., 260.), (10., 285.), (0., 300.)], sharp: vec![false; 11], strips: 16, rings: 20, thickness: 2, ..base.clone() }),
    ]);
    let only_v = std::env::var("ONLY_V").is_ok();
    for (name, s) in list {
        if only_v && !name.starts_with('V') { continue; }
        match shell_bytes(&s, name) {
            Ok((r, bytes)) => {
                std::fs::write(out.join(format!("{name}.brz")), bytes).unwrap();
                // panel corners for the contact sheet
                let mut q = String::new();
                for p in r.panels.iter().filter(|p| !p.shape.is_empty()) {
                    let k = s.colours[p.colour % s.colours.len()];
                    let mut sh = p.shape.clone();
                    while sh.len() < 4 { sh.push(*sh.last().unwrap()); }
                    for (lx, ly) in sh {
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
