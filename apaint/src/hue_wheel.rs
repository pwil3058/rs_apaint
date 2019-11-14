// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Cartesian, Point};
use colour_math::{ColourComponent, ColourInterface, ScalarAttribute};

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

pub trait DrawShapeForAttr<F>: XYForAttribute<F>
where
    F: ColourComponent + ShapeConsts,
{
    fn draw_shape_for_attr(
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
