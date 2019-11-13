// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use crate::drawing::{Cartesian, Dirn, Point};
use crate::ColouredItem;
use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};
use normalised_angles::Degrees;

pub trait Graticule<F>
where
    F: ColourComponent,
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
            let angle = hue.angle();
            let start: Point<F> = (angle, start_ring).into();
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ShapeType {
    Circle,
    Diamond,
    Square,
    Equilateral,
    BackSight,
}

pub struct ColourShape<F: ColourComponent> {
    coloured_item: Rc<dyn ColouredItem<F>>,
    xy: Point<F>,
    outline_rgb: RGB<F>,
    shape_type: ShapeType,
}

impl<F: ColourComponent> ColourShape<F> {
    pub fn new(
        coloured_item: Rc<dyn ColouredItem<F>>,
        scalar_attribute: ScalarAttribute,
        shape_type: ShapeType,
    ) -> ColourShape<F> {
        let xy: Point<F> = if let Some(hue_angle) = coloured_item.hue_angle() {
            let radius = coloured_item.scalar_attribute(scalar_attribute);
            Point::from((hue_angle, radius))
        } else {
            Point {
                x: F::from(-1.05).unwrap(),
                y: F::ONE - F::TWO * coloured_item.value(),
            }
        };
        let outline_rgb = coloured_item.best_foreground_rgb();
        Self {
            coloured_item,
            xy,
            outline_rgb,
            shape_type,
        }
    }

    pub fn coloured_item(&self) -> Rc<dyn ColouredItem<F>> {
        Rc::clone(&self.coloured_item)
    }

    pub fn draw(&self, cartesian: &impl Cartesian<F>) {
        let side = F::from(0.06).unwrap();
        cartesian.set_fill_colour(self.coloured_item.rgb());
        cartesian.set_line_colour(self.outline_rgb);
        cartesian.set_line_width(F::from(0.01).unwrap());
        match self.shape_type {
            ShapeType::Circle => {
                let radius = side / F::TWO;
                cartesian.draw_circle(self.xy, radius, true);
                cartesian.draw_circle(self.xy, radius, false);
            }
            ShapeType::Diamond => {
                cartesian.draw_diamond(self.xy, side, true);
                cartesian.draw_diamond(self.xy, side, false);
            }
            ShapeType::Square => {
                cartesian.draw_square(self.xy, side, true);
                cartesian.draw_square(self.xy, side, false);
            }
            ShapeType::Equilateral => {
                cartesian.draw_equilateral(self.xy, Dirn::Up, side, true);
                cartesian.draw_equilateral(self.xy, Dirn::Up, side, false);
            }
            ShapeType::BackSight => {
                let radius = side / F::TWO;
                cartesian.draw_circle(self.xy, radius, true);
                cartesian.draw_circle(self.xy, radius, false);
                cartesian.draw_plus_sign(self.xy, side);
            }
        }
    }
}

impl<F: ColourComponent> Graticule<F> for Vec<ColourShape<F>> {
    fn draw(&self, cartesian: &impl Cartesian<F>) {
        self.draw_graticule(cartesian);
        for shape in self.iter() {
            shape.draw(cartesian);
        }
    }
}
