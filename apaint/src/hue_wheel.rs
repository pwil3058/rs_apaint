// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Cartesian, Point};
use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};
use float_plus::FloatPlus;
use normalised_angles::Degrees;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
enum CachedPoint<F: ColourComponent + ShapeConsts> {
    Hued(Point<F>),
    Grey(Point<F>),
}

#[derive(Debug)]
pub struct ColouredShape<F: ColourComponent + ShapeConsts> {
    id: String,
    rgb: RGB<F>,
    cached_point: CachedPoint<F>,
    tooltip_text: String,
    shape: Shape,
}

impl<F: ColourComponent + ShapeConsts> ColouredShape<F> {
    pub fn new(rgb: RGB<F>, id: &str, tooltip_text: &str, shape: Shape) -> Self {
        let cached_point = if let Some(hue_angle) = rgb.hue_angle() {
            CachedPoint::Hued(Point::from((hue_angle, F::ONE)))
        } else {
            CachedPoint::Grey(Point {
                x: F::from(-1.05).unwrap(),
                y: F::ONE - F::TWO * rgb.value(),
            })
        };
        Self {
            id: id.to_string(),
            rgb,
            cached_point,
            tooltip_text: tooltip_text.to_string(),
            shape,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn xy(&self, scalar_attribute: ScalarAttribute) -> Point<F> {
        match self.cached_point {
            CachedPoint::Hued(point) => point * self.rgb.scalar_attribute(scalar_attribute),
            CachedPoint::Grey(point) => point,
        }
    }

    pub fn draw_shape(&self, scalar_attribute: ScalarAttribute, cartesian: &impl Cartesian<F>) {
        cartesian.set_fill_colour(self.rgb);
        cartesian.set_line_colour(self.rgb.best_foreground_rgb());
        cartesian.set_line_width(F::from(0.01).unwrap());
        let xy = self.xy(scalar_attribute);
        match self.shape {
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

    fn proximity_to(&self, point: Point<F>, scalar_attribute: ScalarAttribute) -> Proximity<F> {
        let delta = self.xy(scalar_attribute) - point;
        let distance = delta.hypot();
        // TODO: finish implementing enclosed component of proximity_to()
        match self.shape {
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

impl<F: ColourComponent + ShapeConsts> From<&RGB<F>> for ColouredShape<F> {
    fn from(rgb: &RGB<F>) -> Self {
        let id = format!("ID: {}", rgb.pango_string());
        let tooltip_text = format!("RGB: {}", id);
        ColouredShape::new(*rgb, &id, &tooltip_text, Shape::Circle)
    }
}

pub trait Graticule<F: ColourComponent + ShapeConsts> {
    fn draw_rings(num_rings: u32, cartesian: &impl Cartesian<F>) {
        cartesian.set_line_width(F::from(0.01).unwrap());
        cartesian.set_line_colour(RGB::WHITE); // * F::from(0.25).unwrap());
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
        cartesian.set_background_colour(RGB::WHITE * F::HALF);
        Self::draw_spokes(F::from(0.1).unwrap(), cartesian);
        Self::draw_rings(10, cartesian);
    }
}

pub struct HueWheel<F: ColourComponent + ShapeConsts> {
    shapes: Vec<ColouredShape<F>>,
    target: Option<ColouredShape<F>>,
}

impl<F: ColourComponent + ShapeConsts> Graticule<F> for HueWheel<F> {}

impl<F: ColourComponent + ShapeConsts> HueWheel<F> {
    pub fn new() -> Self {
        Self {
            shapes: vec![],
            target: None,
        }
    }

    pub fn draw(&self, scalar_attribute: ScalarAttribute, cartesian: &impl Cartesian<F>) {
        self.draw_graticule(cartesian);
        for shape in self.shapes.iter() {
            shape.draw_shape(scalar_attribute, cartesian);
        }
        if let Some(ref target) = self.target {
            target.draw_shape(scalar_attribute, cartesian)
        }
    }

    fn nearest_to(
        &self,
        point: Point<F>,
        scalar_attribute: ScalarAttribute,
    ) -> Option<(&ColouredShape<F>, Proximity<F>)> {
        let mut nearest: Option<(&ColouredShape<F>, Proximity<F>)> = None;
        for shape in self.shapes.iter() {
            let proximity = shape.proximity_to(point, scalar_attribute);
            if let Some((_, nearest_so_far)) = nearest {
                if proximity < nearest_so_far {
                    nearest = Some((shape, proximity));
                }
            } else {
                nearest = Some((shape, proximity));
            }
        }
        nearest
    }

    pub fn item_at_point(
        &self,
        point: Point<F>,
        scalar_attribute: ScalarAttribute,
    ) -> Option<&ColouredShape<F>> {
        if let Some((shape, proximity)) = self.nearest_to(point, scalar_attribute) {
            if let Proximity::Enclosed(_) = proximity {
                return Some(shape);
            }
        };
        None
    }

    pub fn tooltip_for_point(
        &self,
        point: Point<F>,
        scalar_attribute: ScalarAttribute,
    ) -> Option<String> {
        if let Some((shape, _)) = self.nearest_to(point, scalar_attribute) {
            return Some(shape.tooltip_text.to_string());
        }
        None
    }

    pub fn add_item(&mut self, coloured_item: ColouredShape<F>) {
        self.shapes.push(coloured_item)
    }
}

impl ShapeConsts for f64 {
    const SIN_45: Self = f64::SQRT_2 / 2.0;
    const SHAPE_SIDE: Self = 0.06;
    const SHAPE_HALF_SIDE: Self = Self::SHAPE_SIDE / 2.0;
    const SHAPE_RADIUS: Self = Self::SHAPE_HALF_SIDE;
}

impl ShapeConsts for f32 {
    const SIN_45: Self = f32::SQRT_2 / 2.0;
    const SHAPE_SIDE: Self = 0.06;
    const SHAPE_HALF_SIDE: Self = Self::SHAPE_SIDE / 2.0;
    const SHAPE_RADIUS: Self = Self::SHAPE_HALF_SIDE;
}
