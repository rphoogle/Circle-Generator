//! Writes an in-game test pack (.brz files) plus pieces.json for the preview sheet.
use circlegen::*;
use std::io::Write;
fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    std::fs::create_dir_all(&out).unwrap();
    use BrickStyle::*; use Shape::*; use Kind::*; use Fill::*;
    // (file, style, shape, kind, inverse, fill, width, length, hole_w, hole_l, wall, height, colour)
    let tests: Vec<(&str, BrickStyle, Shape, Kind, bool, Fill, i32, i32, i32, i32, i32, i32, (u8,u8,u8))> = vec![
        ("O01_Oval_Solid_Strips_80x160M", Micro, Full, Solid, false, Strips, 80, 160, 0, 0, 0, 18, (255,255,255)),
        ("O02_Oval_Solid_RectangleCentre_80x160M", Micro, Full, Solid, false, Blocks, 80, 160, 0, 0, 0, 18, (255,255,255)),
        ("O03_Oval_Solid_RectangleCentre_81x130M_odd", Micro, Full, Solid, false, Blocks, 81, 130, 0, 0, 0, 18, (240,200,60)),
        ("O04_Oval_Half_RectangleCentre_80x160M", Micro, Half, Solid, false, Blocks, 80, 160, 0, 0, 0, 18, (240,200,60)),
        ("O05_Oval_Hollow_80x160M", Micro, Full, Hollow, false, Strips, 80, 160, 0, 0, 0, 18, (90,170,240)),
        ("O06_Oval_SolidInverse_Strips_80x160M_wall5", Micro, Full, Solid, true, Strips, 80, 160, 0, 0, 5, 18, (90,170,240)),
        ("D01_Doughnut_Default_24studs_hole16_FourWay", Default, Full, Ring, false, Blocks, 24, 24, 16, 16, 0, 3, (120,200,120)),
        ("D02_Doughnut_80M_hole50_FourWay", Micro, Full, Ring, false, Blocks, 80, 80, 50, 50, 0, 18, (230,110,90)),
        ("D03_Doughnut_80M_hole50_Strips", Micro, Full, Ring, false, Strips, 80, 80, 50, 50, 0, 18, (230,110,90)),
        ("D04_Doughnut_80M_hole68_thin_FourWay", Micro, Full, Ring, false, Blocks, 80, 80, 68, 68, 0, 18, (230,110,90)),
        ("D05_Doughnut_80M_hole76_verythin", Micro, Full, Ring, false, Blocks, 80, 80, 76, 76, 0, 18, (230,110,90)),
        ("D06_QuarterDoughnut_90degBend_80M_hole60", Micro, Quarter, Ring, false, Blocks, 80, 80, 60, 60, 0, 18, (180,140,220)),
        ("D07_HalfDoughnut_UBend_Tile_32studs_hole24", Tile, Half, Ring, false, Blocks, 32, 32, 24, 24, 0, 3, (120,200,120)),
        ("D08_OvalDoughnut_80x160M_band6", Micro, Full, Ring, false, Blocks, 80, 160, 68, 148, 0, 18, (180,140,220)),
        ("D09_InverseDoughnut_80M_hole50_wall5", Micro, Full, Ring, true, Blocks, 80, 80, 50, 50, 5, 18, (180,140,220)),
    ];
    let mut index = std::fs::File::create(out.join("pieces.json")).unwrap();
    writeln!(index, "[").unwrap();
    for (n, t) in tests.iter().enumerate() {
        let (file, style, shape, kind, inv, fill, w, len, hw, hl, wall, h, col) = *t;
        let l = Layout { shape, kind, inverse: inv, fill, width: w, length: len, hole_w: hw, hole_l: hl, wall };
        let s = Settings { layout: l, style, height: h, color: col, material: "BMC_Plastic".into(), name: file.to_string() };
        let b = write_brz(&s, &out.join(format!("{file}.brz"))).expect(file);
        let r = check::check(&l, &b.pieces);
        println!("{file}: {} bricks ({} wedges) check ok={} {:?}", b.bricks.len(), b.wedges, r.ok(), r);
        let polys: Vec<String> = b.pieces.iter().map(|p| {
            let pts: Vec<String> = p.polygon().iter().map(|(x, y)| format!("[{x},{y}]")).collect();
            format!("{{\"w\":{},\"p\":[{}]}}", p.is_wedge(), pts.join(","))
        }).collect();
        writeln!(index, "{{\"file\":\"{file}\",\"bricks\":{},\"wedges\":{},\"stud\":{},\"d\":{w},\"pieces\":[{}]}}{}",
            b.bricks.len(), b.wedges, style.is_stud(), polys.join(","), if n + 1 < tests.len() { "," } else { "" }).unwrap();
    }
    writeln!(index, "]").unwrap();
}
