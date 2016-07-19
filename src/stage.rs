use ::game::Point;

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Stage {
    pub title:         String,
    pub platforms:     Vec<Platform>,
    pub lower_bounds:  Point,
    pub higher_bounds: Point,
    pub spawn_points:  Vec<Point>,
}

impl Stage {
    pub fn base() -> Stage {
        let main_platform = Platform {
            x: 0.0,
            y: 0.0,
            w: 150.0,
            h: 10.0,
            pass_through: false,
        };

        let second_platform = Platform {
            x: 50.0,
            y: 50.0,
            w: 50.0,
            h: 2.5,
            pass_through: true,
        };

        Stage {
            title:         "Base Stage".to_string(),
            platforms:     vec!(main_platform, second_platform),
            lower_bounds:  Point {x:-200.0, y:-200.0},
            higher_bounds: Point {x: 200.0, y: 200.0},
            spawn_points:  vec!(
                Point{x: -50.0, y: 50.0}, Point{x: 50.0, y: 50.0},
                Point{x: -50.0, y: 80.0}, Point{x: 50.0, y: 80.0},
            ),
        }
    }
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Platform {
    pub x:            f32,
    pub y:            f32,
    pub w:            f32,
    pub h:            f32,
    pub pass_through: bool,
}
