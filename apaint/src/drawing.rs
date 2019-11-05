// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use colour_math::{ColourComponent, RGB};
use float_plus::FloatPlus;
use normalised_angles::{Angle, Degrees, DegreesConst, RadiansConst};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point<F: FloatPlus> {
    pub x: F,
    pub y: F,
}

impl<F: FloatPlus> Point<F> {
    pub fn hypot(&self) -> F {
        self.x.hypot(self.y)
    }
}

impl<F: FloatPlus + DegreesConst + RadiansConst> Point<F> {
    pub fn angle(&self) -> Angle<F> {
        if let Some(degrees) = Degrees::atan2(self.y, self.x) {
            degrees.into()
        } else {
            Degrees::DEG_0.into()
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

impl<F: FloatPlus + DegreesConst + RadiansConst> From<(Angle<F>, F)> for Point<F> {
    fn from(polar: (Angle<F>, F)) -> Point<F> {
        Point {
            x: polar.1 * polar.0.cos(),
            y: polar.1 * polar.0.sin(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size<F: FloatPlus> {
    pub width: F,
    pub height: F,
}

/// Direction in which to draw isosceles triangle
pub enum Dirn {
    Down,
    Up,
    Right,
    Left,
}

pub trait Draw<F: ColourComponent + DegreesConst + RadiansConst> {
    fn size(&self) -> Size<F>;
    fn draw_circle(&self, centre: Point<F>, radius: F, filled: bool);
    fn draw_diamond(&self, centre: Point<F>, side_length: F, filled: bool);
    fn draw_line(&self, line: &[Point<F>]);
    fn draw_polygon(&self, polygon: &[Point<F>], filled: bool);
    fn draw_square(&self, centre: Point<F>, side_length: F, filled: bool);
    fn draw_isosceles(&self, position: Point<F>, dirn: Dirn, size: F, filled: bool);
    fn move_to_point(&self, point: Point<F>);
    fn line_to_point(&self, point: Point<F>);
    fn set_line_colour(&self, rgb: RGB<F>);
    fn set_fill_colour(&self, rgb: RGB<F>);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
