# Circle Generator for Brickadia

Makes smooth circles as Brickadia prefabs (.brz): full, half and quarter;
solid, hollow and inverse (a round hole); in micro bricks or in studs
(Default, Tile, Smooth). Built on [brdb](https://github.com/brickadia-community/brdb).

## Getting the .exe
GitHub builds it automatically: open the **Actions** tab, click the latest
"Build Windows .exe" run, and download **CircleGenerator-windows** under
*Artifacts*. Windows may show a SmartScreen warning because the program is
unsigned: choose *More info* then *Run anyway*.

## How it is organised
- `src/geometry.rs` – where every brick goes (the shape rules)
- `src/lib.rs` – turns the layout into bricks and a .brz prefab
- `src/check.rs` – self-check for gaps and overlaps, run before every save
- `src/main.rs` – the window
- `src/bin/sweep.rs`, `src/bin/testpack.rs` – test tools

Units: 1 M = one micro brick (0.2 stud); 5 M = 1 stud; 2 M = 1 plate; 6 M = 1 brick.
Largest single brick: 800 studs (4000 M). Fonts: DejaVu (see assets/DejaVu-LICENSE.txt).
