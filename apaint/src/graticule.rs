// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use crate::drawing::{Cartesian, Point};
use crate::{ColouredItem, TooltipText};
use colour_math::{ColourComponent, ColourInterface, RGB};
use normalised_angles::{Angle, Degrees, DegreesConst, RadiansConst};

pub trait Graticule<F>
where
    F: ColourComponent + DegreesConst + RadiansConst,
{
    fn draw(&self, cartesian: &impl Cartesian<F>) {
        self.draw_graticule(cartesian);
    }

    fn draw_rings(&self, num_rings: u32, cartesian: &impl Cartesian<F>) {
        cartesian.set_line_width(F::from(0.01).unwrap());
        cartesian.set_line_colour(RGB::WHITE * F::HALF);
        let divisor = F::from_u32(num_rings).unwrap();
        let centre = Point::<F>::default();
        for num in 1..num_rings + 1 {
            let radius: F = F::from(num).unwrap() / divisor;
            cartesian.draw_circle(centre, radius, false);
        }
    }

    fn draw_spokes(&self, start_ring: F, cartesian: &impl Cartesian<F>) {
        cartesian.set_line_width(F::from(0.015).unwrap());
        let mut hue = RGB::<F>::RED.hue().unwrap();
        for _ in 0..13 {
            cartesian.set_line_colour(hue.max_chroma_rgb());
            let angle: Angle<F> = hue.angle().into();
            let start: Point<F> = (angle.clone(), start_ring).into();
            let end: Point<F> = (angle, F::ONE).into();
            cartesian.draw_line(&[start, end]);
            hue = hue + Degrees::DEG_30;
        }
    }

    fn draw_graticule(&self, cartesian: &impl Cartesian<F>) {
        self.draw_spokes(F::from(0.1).unwrap(), cartesian);
        self.draw_rings(10, cartesian);
    }

    fn tooltip_for_point(&self, _point: Point<F>) -> Option<String> {
        None
    }

    fn item_at_point(&self, _point: Point<F>) -> Option<Rc<dyn ColouredItem<F>>> {
        None
    }
}
