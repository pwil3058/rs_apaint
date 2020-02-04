// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use colour_math::{ColourComponent, RGB};
use float_plus::FloatPlus;
use normalised_angles::Degrees;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<F: FloatPlus> {
    pub x: F,
    pub y: F,
}

impl<F: FloatPlus> Point<F> {
    pub fn hypot(&self) -> F {
        self.x.hypot(self.y)
    }
}

impl<F: ColourComponent> Point<F> {
    pub fn angle(&self) -> Degrees<F> {
        if let Some(degrees) = Degrees::atan2(self.y, self.x) {
            degrees
        } else {
            Degrees::DEG_0
        }
    }
}

impl<F: FloatPlus> std::default::Default for Point<F> {
    fn default() -> Point<F> {
        Point {
            x: F::ZERO,
            y: F::ZERO,
        }
    }
}

impl<F: FloatPlus> From<[F; 2]> for Point<F> {
    fn from(array: [F; 2]) -> Self {
        Self {
            x: array[0],
            y: array[1],
        }
    }
}

impl<F: FloatPlus> From<(F, F)> for Point<F> {
    fn from(tuple: (F, F)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl<F: ColourComponent> From<(Degrees<F>, F)> for Point<F> {
    fn from(polar: (Degrees<F>, F)) -> Point<F> {
        Point {
            x: polar.1 * polar.0.cos(),
            y: polar.1 * polar.0.sin(),
        }
    }
}

impl<F: ColourComponent> std::ops::Sub for Point<F> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<F: ColourComponent> std::ops::Add for Point<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<F: ColourComponent> std::ops::Add<Size<F>> for Point<F> {
    type Output = Self;

    fn add(self, size: Size<F>) -> Self {
        Self {
            x: self.x + size.width,
            y: self.y + size.height,
        }
    }
}

impl<F: ColourComponent> std::ops::Mul<F> for Point<F> {
    type Output = Self;

    fn mul(self, scalar: F) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size<F: FloatPlus> {
    pub width: F,
    pub height: F,
}

impl<F: FloatPlus> Size<F> {
    pub fn centre(&self) -> Point<F> {
        (self.width / F::TWO, self.height / F::TWO).into()
    }
}

/// Direction in which to draw isosceles triangle
pub enum Dirn {
    Down,
    Up,
    Right,
    Left,
}

pub enum TextPosn<F: FloatPlus> {
    TopLeftCorner(Point<F>),
    TopRightCorner(Point<F>),
    BottomLeftCorner(Point<F>),
    BottomRightCorner(Point<F>),
    Centre(Point<F>),
}

pub trait Draw<F: ColourComponent> {
    fn size(&self) -> Size<F>;
    fn draw_circle(&self, centre: Point<F>, radius: F, fill: bool);
    fn draw_line(&self, line: &[Point<F>]);
    fn draw_polygon(&self, polygon: &[Point<F>], fill: bool);
    fn draw_text(&self, text: &str, posn: TextPosn<F>, font_size: F);
    fn set_background_colour(&self, rgb: RGB<F>);
    fn set_line_width(&self, width: F);
    fn set_line_colour(&self, rgb: RGB<F>);
    fn set_fill_colour(&self, rgb: RGB<F>);
    fn set_text_colour(&self, rgb: RGB<F>);
    fn paint_linear_gradient(&self, posn: Point<F>, size: Size<F>, colour_stops: &[(RGB<F>, F)]);
    fn insert_png(&self, posn: Point<F>, reader: &mut impl std::io::Read);

    fn draw_diamond(&self, centre: Point<F>, side_length: F, fill: bool) {
        let dist = side_length / F::SQRT_2;
        let points = vec![
            Point {
                x: centre.x,
                y: centre.y + dist,
            },
            Point {
                x: centre.x + dist,
                y: centre.y,
            },
            Point {
                x: centre.x,
                y: centre.y - dist,
            },
            Point {
                x: centre.x - dist,
                y: centre.y,
            },
        ];
        self.draw_polygon(&points, fill);
    }

    fn draw_square(&self, centre: Point<F>, side_length: F, fill: bool) {
        let half_side = side_length * F::HALF;
        let points = vec![
            Point {
                x: centre.x - half_side,
                y: centre.y - half_side,
            },
            Point {
                x: centre.x - half_side,
                y: centre.y + half_side,
            },
            Point {
                x: centre.x + half_side,
                y: centre.y + half_side,
            },
            Point {
                x: centre.x + half_side,
                y: centre.y - half_side,
            },
        ];
        self.draw_polygon(&points, fill);
    }

    fn draw_equilateral(&self, centre: Point<F>, dirn: Dirn, side_length: F, fill: bool) {
        let half_base = side_length * F::HALF;
        let half_height = side_length * F::SQRT_3 / F::FOUR;
        let points = match dirn {
            Dirn::Up => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y - half_height,
                },
            ],
            Dirn::Down => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y + half_height,
                },
            ],
            Dirn::Right => vec![
                Point {
                    x: centre.x - half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y,
                },
            ],
            Dirn::Left => vec![
                Point {
                    x: centre.x + half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y,
                },
            ],
        };
        self.draw_polygon(&points, fill);
    }

    fn draw_isosceles(&self, centre: Point<F>, dirn: Dirn, base: F, height: F, fill: bool) {
        let half_base = base * F::HALF;
        let half_height = height * F::HALF;
        let points = match dirn {
            Dirn::Up => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y - half_height,
                },
            ],
            Dirn::Down => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y + half_height,
                },
            ],
            Dirn::Right => vec![
                Point {
                    x: centre.x - half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y,
                },
            ],
            Dirn::Left => vec![
                Point {
                    x: centre.x + half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y,
                },
            ],
        };
        self.draw_polygon(&points, fill);
    }
}

pub trait Cartesian<F: ColourComponent> {
    fn draw_circle(&self, centre: Point<F>, radius: F, fill: bool);
    fn draw_line(&self, line: &[Point<F>]);
    fn draw_polygon(&self, polygon: &[Point<F>], fill: bool);
    fn set_line_width(&self, width: F);
    fn set_background_colour(&self, rgb: RGB<F>);
    fn set_line_colour(&self, rgb: RGB<F>);
    fn set_fill_colour(&self, rgb: RGB<F>);

    fn draw_diamond(&self, centre: Point<F>, side_length: F, fill: bool) {
        let dist = side_length / F::SQRT_2;
        let points = vec![
            Point {
                x: centre.x,
                y: centre.y + dist,
            },
            Point {
                x: centre.x + dist,
                y: centre.y,
            },
            Point {
                x: centre.x,
                y: centre.y - dist,
            },
            Point {
                x: centre.x - dist,
                y: centre.y,
            },
        ];
        self.draw_polygon(&points, fill);
    }

    fn draw_square(&self, centre: Point<F>, side_length: F, fill: bool) {
        let half_side = side_length * F::HALF;
        let points = vec![
            Point {
                x: centre.x - half_side,
                y: centre.y - half_side,
            },
            Point {
                x: centre.x - half_side,
                y: centre.y + half_side,
            },
            Point {
                x: centre.x + half_side,
                y: centre.y + half_side,
            },
            Point {
                x: centre.x + half_side,
                y: centre.y - half_side,
            },
        ];
        self.draw_polygon(&points, fill);
    }

    fn draw_equilateral(&self, centre: Point<F>, dirn: Dirn, side_length: F, fill: bool) {
        let half_base = side_length * F::HALF;
        let half_height = side_length * F::SQRT_3 / F::FOUR;
        let points = match dirn {
            Dirn::Up => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y - half_height,
                },
            ],
            Dirn::Down => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y + half_height,
                },
            ],
            Dirn::Right => vec![
                Point {
                    x: centre.x - half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y,
                },
            ],
            Dirn::Left => vec![
                Point {
                    x: centre.x + half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y,
                },
            ],
        };
        self.draw_polygon(&points, fill);
    }

    fn draw_isosceles(&self, centre: Point<F>, dirn: Dirn, base: F, height: F, fill: bool) {
        let half_base = base * F::HALF;
        let half_height = height * F::HALF;
        let points = match dirn {
            Dirn::Up => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y - half_height,
                },
            ],
            Dirn::Down => vec![
                Point {
                    x: centre.x - half_base,
                    y: centre.y + half_height,
                },
                Point {
                    x: centre.x,
                    y: centre.y - half_height,
                },
                Point {
                    x: centre.x + half_base,
                    y: centre.y + half_height,
                },
            ],
            Dirn::Right => vec![
                Point {
                    x: centre.x - half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y,
                },
            ],
            Dirn::Left => vec![
                Point {
                    x: centre.x + half_height,
                    y: centre.y - half_base,
                },
                Point {
                    x: centre.x + half_height,
                    y: centre.y + half_base,
                },
                Point {
                    x: centre.x - half_height,
                    y: centre.y,
                },
            ],
        };
        self.draw_polygon(&points, fill);
    }

    fn draw_plus_sign(&self, centre: Point<F>, side_length: F) {
        let half_side = side_length * F::HALF;
        let points = vec![
            Point {
                x: centre.x,
                y: centre.y - half_side,
            },
            Point {
                x: centre.x,
                y: centre.y + half_side,
            },
        ];
        self.draw_line(&points);
        let points = vec![
            Point {
                x: centre.x - half_side,
                y: centre.y,
            },
            Point {
                x: centre.x + half_side,
                y: centre.y,
            },
        ];
        self.draw_line(&points);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
