// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Cartesian, Point};
use colour_math::{ColourComponent, ColourInterface, RGB};
use normalised_angles::{Angle, Degrees, DegreesConst, RadiansConst};

#[derive(Default)]
pub struct Graticule<F>
where
    F: ColourComponent + DegreesConst + RadiansConst,
{
    phantom: std::marker::PhantomData<F>,
}

impl<F> Graticule<F>
where
    F: ColourComponent + DegreesConst + RadiansConst,
{
    pub fn draw(&self, cartesian: &impl Cartesian<F>) {
        self.draw_graticule(cartesian);
    }

    fn draw_graticule(&self, cartesian: &impl Cartesian<F>) {
        let num_rings = 10;
        let divisor = F::from_i32(num_rings).unwrap();
        let centre = Point::<F>::default();
        cartesian.set_line_width(F::from(0.01).unwrap());
        cartesian.set_line_colour(RGB::WHITE * F::HALF);
        for num in 1..num_rings + 1 {
            let radius: F = F::from(num).unwrap() / divisor;
            cartesian.draw_circle(centre, radius, false);
        }
        cartesian.set_line_width(F::from(0.015).unwrap());
        let mut hue = RGB::<F>::RED.hue().unwrap();
        for _ in 0..13 {
            cartesian.set_line_colour(hue.max_chroma_rgb());
            let angle: Angle<F> = hue.angle().into();
            let end: Point<F> = (angle, F::ONE).into();
            cartesian.draw_line(&[centre, end]);
            hue = hue + Degrees::DEG_30;
        }
    }
}
