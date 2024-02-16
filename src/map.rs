use super::{MAP_HEIGHT, MAP_WIDTH, MAX_X, MAX_Y, MIN_X, MIN_Y};
use rltk::{RandomNumberGenerator, Rltk, RGB};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_index(x: i32, y: i32) -> usize {
    ((y * (MAP_WIDTH as i32)) + x) as usize
}

pub fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; MAP_WIDTH * MAP_HEIGHT];

    // Make the boundaries walls
    for x in 0..(MAP_WIDTH as i32) {
        map[xy_index(x, MIN_Y)] = TileType::Wall;
        map[xy_index(x, MAX_Y)] = TileType::Wall;
    }
    for y in 0..(MAP_HEIGHT as i32) {
        map[xy_index(MIN_X, y)] = TileType::Wall;
        map[xy_index(MAX_X, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = RandomNumberGenerator::new();

    for _i in 0..200 {
        let x = rng.roll_dice(1, MAX_X);
        let y = rng.roll_dice(1, MAX_Y);
        let index = xy_index(x, y);

        // not starting position
        if index != xy_index(40, 25) {
            map[index] = TileType::Wall;
        }
    }

    map
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;

    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
        }

        // Move the coordinates
        x += 1;
        if x > MAX_X {
            x = MIN_X;
            y += 1;
        }
    }
}
