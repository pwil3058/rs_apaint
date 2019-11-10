// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Cartesian, Point};
use colour_math::{ColourComponent, RGB};
use normalised_angles::{DegreesConst, RadiansConst};

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
        cartesian.set_line_width(F::from(0.01).unwrap());
        cartesian.set_line_colour(RGB::RED);
        cartesian.draw_line(&[
            Point {
                x: -F::ONE,
                y: F::ZERO,
            },
            Point {
                x: F::ONE,
                y: F::ZERO,
            },
        ]);
        cartesian.draw_line(&[
            Point {
                x: F::ZERO,
                y: F::ONE,
            },
            Point {
                x: F::ZERO,
                y: -F::ONE,
            },
        ]);
        cartesian.draw_circle(
            Point {
                x: F::ZERO,
                y: F::ZERO,
            },
            F::ONE,
            false,
        );
        cartesian.draw_diamond(
            Point {
                x: F::HALF,
                y: F::HALF,
            },
            F::from(0.1).unwrap(),
            false,
        );
    }
}
