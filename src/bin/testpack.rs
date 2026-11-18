use circlegen::*;
use std::io::Write;
fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    std::fs::create_dir_all(&out).unwrap();
    use BrickStyle::*; use Shape::*; use Kind::*; use Fill::*;
    // (file, style, shape, kind, inverse, fill, diameter, wall, height, colour)
    let tests: Vec<(&str, BrickStyle, Shape, Kind, bool, Fill, i32, i32, i32, (u8,u8,u8))> = vec![
        ("T01_Micro_Full_Solid_SquareCentre_80M", Micro, Full, Solid, false, Blocks, 80, 0, 18, (255,255,255)),
        ("T02_Micro_Full_Hollow_80M", Micro, Full, Hollow, false, Strips, 80, 0, 18, (255,255,255)),
        ("T03_Micro_Full_Solid_Strips_81M_odd", Micro, Full, Solid, false, Strips, 81, 0, 18, (240,200,60)),
        ("T04_Micro_Full_Solid_SquareCentre_81M_odd", Micro, Full, Solid, false, Blocks, 81, 0, 18, (240,200,60)),
        ("T05_Micro_Half_Solid_SquareCentre_80M", Micro, Half, Solid, false, Blocks, 80, 0, 18, (90,170,240)),
        ("T06_Micro_Quarter_Solid_SquareCentre_80M", Micro, Quarter, Solid, false, Blocks, 80, 0, 18, (90,170,240)),
        ("T07_Micro_Full_HollowInverse_80M", Micro, Full, Hollow, true, Strips, 80, 0, 18, (230,110,90)),
        ("T08_Micro_Full_SolidInverse_CornerBlocks_80M_wall5", Micro, Full, Solid, true, Blocks, 80, 5, 18, (230,110,90)),
        ("T09_Micro_Quarter_SolidInverse_CornerBlocks_80M_wall5", Micro, Quarter, Solid, true, Blocks, 80, 5, 18, (230,110,90)),
        ("T10_Default_Full_Solid_SquareCentre_16studs_9plates", Default, Full, Solid, false, Blocks, 16, 0, 9, (120,200,120)),
        ("T11_Tile_Full_Solid_SquareCentre_48studs_3plates", Tile, Full, Solid, false, Blocks, 48, 0, 3, (120,200,120)),
        ("T12_Smooth_Half_SolidInverse_Strips_24studs_wall1_3plates", Smooth, Half, Solid, true, Strips, 24, 1, 3, (120,200,120)),
        ("T13_Micro_Full_Solid_Strips_520M_bigsplit", Micro, Full, Solid, false, Strips, 520, 0, 6, (180,140,220)),
    ];
    let mut index = std::fs::File::create(out.join("pieces.json")).unwrap();
    writeln!(index, "[").unwrap();
    for (n, (file, style, shape, kind, inv, fill, d, wall, h, col)) in tests.iter().enumerate() {
        let l = Layout { shape: *shape, kind: *kind, inverse: *inv, fill: *fill, diameter: *d, wall: *wall };
        let s = Settings { layout: l, style: *style, height: *h, color: *col, material: "BMC_Plastic".into(), name: file.to_string() };
        let b = write_brz(&s, &out.join(format!("{file}.brz"))).expect(file);
        let r = check::check(&l, &b.pieces);
        println!("{file}: {} bricks ({} wedges) check ok={}", b.bricks.len(), b.wedges, r.ok());
        let polys: Vec<String> = b.pieces.iter().map(|p| {
            let pts: Vec<String> = p.polygon().iter().map(|(x,y)| format!("[{x},{y}]")).collect();
            format!("{{\"w\":{},\"p\":[{}]}}", p.is_wedge(), pts.join(","))
        }).collect();
        writeln!(index, "{{\"file\":\"{file}\",\"bricks\":{},\"wedges\":{},\"stud\":{},\"d\":{d},\"pieces\":[{}]}}{}",
            b.bricks.len(), b.wedges, style.is_stud(), polys.join(","), if n+1<tests.len() {","} else {""}).unwrap();
    }
    writeln!(index, "]").unwrap();
}
