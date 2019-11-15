// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Cartesian, Point};
use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};
use float_plus::FloatPlus;
use normalised_angles::Degrees;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Shape {
    Circle,
    Diamond,
    Square,
    BackSight,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Proximity<F: ColourComponent + PartialOrd> {
    Enclosed(F),
    NotEnclosed(F),
}

impl<F: ColourComponent> std::cmp::PartialOrd for Proximity<F> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Self::Enclosed(mine) => match other {
                Self::Enclosed(other) => mine.partial_cmp(other),
                Self::NotEnclosed(_) => Some(std::cmp::Ordering::Less),
            },
            Self::NotEnclosed(mine) => match other {
                Self::Enclosed(_) => Some(std::cmp::Ordering::Greater),
                Self::NotEnclosed(other) => mine.partial_cmp(other),
            },
        }
    }
}

pub trait ShapeConsts: ColourComponent {
    const SIN_45: Self;
    const SHAPE_SIDE: Self;
    const SHAPE_HALF_SIDE: Self;
    const SHAPE_RADIUS: Self;
}

pub trait XYForAttribute<F: ColourComponent + ShapeConsts>: ColourInterface<F> {
    fn xy_for_attribute(&self, scalar_attribute: ScalarAttribute) -> Point<F> {
        if let Some(hue_angle) = self.hue_angle() {
            let radius = self.scalar_attribute(scalar_attribute);
            Point::from((hue_angle, radius))
        } else {
            Point {
                x: F::from(-1.05).unwrap(),
                y: F::ONE - F::TWO * self.value(),
            }
        }
    }

    fn proximity_to(
        &self,
        point: Point<F>,
        shape: Shape,
        scalar_attribute: ScalarAttribute,
    ) -> Proximity<F> {
        let delta = self.xy_for_attribute(scalar_attribute) - point;
        let distance = delta.hypot();
        // TODO: finish implementing enclosed component of proximity_to()
        match shape {
            Shape::Circle => {
                if distance < F::SHAPE_RADIUS {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
            Shape::Square => {
                let x = delta.x.abs();
                let y = delta.y.abs();
                if x < F::SHAPE_HALF_SIDE && y < F::SHAPE_HALF_SIDE {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
            Shape::Diamond => {
                // Rotate 45 degrees
                let x = ((delta.x - delta.y) * F::SIN_45).abs();
                let y = ((delta.x + delta.y) * F::SIN_45).abs();
                if x < F::SHAPE_HALF_SIDE && y < F::SHAPE_HALF_SIDE {
                    Proximity::Enclosed(distance)
                } else {
                    Proximity::NotEnclosed(distance)
                }
            }
            _ => Proximity::NotEnclosed(distance),
        }
    }
}

impl<F: ColourComponent + ShapeConsts> XYForAttribute<F> for RGB<F> {}

pub trait DrawShapeForAttr<F>: XYForAttribute<F>
where
    F: ColourComponent + ShapeConsts,
{
    fn draw_shape_for_attr(&self, scalar_attribute: ScalarAttribute, cartesian: &impl Cartesian<F>);

    fn draw_given_shape_for_attr(
        &self,
        shape: Shape,
        scalar_attribute: ScalarAttribute,
        cartesian: &impl Cartesian<F>,
    ) {
        cartesian.set_fill_colour(self.rgb());
        cartesian.set_line_colour(self.best_foreground_rgb());
        cartesian.set_line_width(F::from(0.01).unwrap());
        let xy = self.xy_for_attribute(scalar_attribute);
        match shape {
            Shape::Circle => {
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, true);
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, false);
            }
            Shape::Diamond => {
                cartesian.draw_diamond(xy, F::SHAPE_SIDE, true);
                cartesian.draw_diamond(xy, F::SHAPE_SIDE, false);
            }
            Shape::Square => {
                cartesian.draw_square(xy, F::SHAPE_SIDE, true);
                cartesian.draw_square(xy, F::SHAPE_SIDE, false);
            }
            Shape::BackSight => {
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, true);
                cartesian.draw_circle(xy, F::SHAPE_RADIUS, false);
                cartesian.draw_plus_sign(xy, F::SHAPE_SIDE);
            }
        }
    }
}

impl<F: ColourComponent + ShapeConsts> DrawShapeForAttr<F> for RGB<F> {
    fn draw_shape_for_attr(
        &self,
        scalar_attribute: ScalarAttribute,
        cartesian: &impl Cartesian<F>,
    ) {
        self.draw_given_shape_for_attr(Shape::Circle, scalar_attribute, cartesian)
    }
}

pub trait Graticule<F: ColourComponent + ShapeConsts> {
    fn draw_rings(num_rings: u32, cartesian: &impl Cartesian<F>) {
        cartesian.set_line_width(F::from(0.01).unwrap());
        cartesian.set_line_colour(RGB::WHITE * F::HALF);
        let divisor = F::from_u32(num_rings).unwrap();
        let centre = Point::<F>::default();
        for num in 1..num_rings + 1 {
            let radius: F = F::from(num).unwrap() / divisor;
            cartesian.draw_circle(centre, radius, false);
        }
    }

    fn draw_spokes(start_ring: F, cartesian: &impl Cartesian<F>) {
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
        Self::draw_spokes(F::from(0.1).unwrap(), cartesian);
        Self::draw_rings(10, cartesian);
    }
}

impl<F: ColourComponent + ShapeConsts, CI: DrawShapeForAttr<F>> Graticule<F> for Vec<CI> {}

pub trait HueWheel<F, CI>: Graticule<F>
where
    F: ColourComponent + ShapeConsts,
    CI: DrawShapeForAttr<F>,
{
    fn draw_all(&self, scalar_attribute: ScalarAttribute, cartesian: &impl Cartesian<F>);

    fn tooltip_for_point(&self, _point: Point<F>) -> Option<String> {
        None
    }

    fn item_at_point(&self, _point: Point<F>) -> Option<CI> {
        None
    }
}

impl<F, CI> HueWheel<F, CI> for Vec<CI>
where
    F: ColourComponent + ShapeConsts,
    CI: DrawShapeForAttr<F>,
{
    fn draw_all(&self, scalar_attribute: ScalarAttribute, cartesian: &impl Cartesian<F>) {
        for item in self.iter() {
            item.draw_shape_for_attr(scalar_attribute, cartesian);
        }
    }
}

impl ShapeConsts for f64 {
    const SIN_45: Self = f64::SQRT_2 / 2.0;
    const SHAPE_SIDE: Self = 0.06;
    const SHAPE_HALF_SIDE: Self = Self::SHAPE_SIDE / 2.0;
    const SHAPE_RADIUS: Self = Self::SHAPE_HALF_SIDE;
}
