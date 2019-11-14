// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::rc::Rc;

use crate::drawing::Dirn;
use crate::{
    drawing::{Cartesian, Point},
    ColouredItem,
};
use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

pub trait Drawable<F: ColourComponent> {
    fn draw(&self, cartesian: &impl Cartesian<F>);
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

pub trait DrawableColourShape<F, CI>: Drawable<F>
where
    F: ColourComponent,
    CI: ColouredItem<F>,
{
    fn proximity_to(&self, _point: Point<F>) -> Proximity<F>;
    fn coloured_item(&self) -> Rc<CI>;
}

pub trait DrawableColourShapes<F, T, CI>
where
    F: ColourComponent,
    T: DrawableColourShape<F, CI> + Sized,
    CI: ColouredItem<F>,
{
    fn iter(&self) -> Box<dyn Iterator<Item = T>>;

    fn nearest_to(&self, point: Point<F>) -> Option<(Rc<CI>, Proximity<F>)> {
        let mut nearest: Option<(Rc<CI>, Proximity<F>)> = None;
        for t in self.iter() {
            let proximity = t.proximity_to(point);
            if let Some((_, nearest_so_far)) = nearest {
                if proximity < nearest_so_far {
                    nearest = Some((t.coloured_item(), proximity));
                }
            } else {
                nearest = Some((t.coloured_item(), proximity));
            }
        }
        nearest
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

pub struct ColourShape<F, CI>
where
    F: ColourComponent,
    CI: ColouredItem<F>,
{
    coloured_item: Rc<CI>,
    xy: Point<F>,
    outline_rgb: RGB<F>,
    shape_type: ShapeType,
}

impl<F, CI> ColourShape<F, CI>
where
    F: ColourComponent,
    CI: ColouredItem<F>,
{
    pub fn new(
        coloured_item: Rc<CI>,
        scalar_attribute: ScalarAttribute,
        shape_type: ShapeType,
    ) -> Self {
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
}

impl<F, CI> Drawable<F> for ColourShape<F, CI>
where
    F: ColourComponent,
    CI: ColouredItem<F>,
{
    fn draw(&self, cartesian: &impl Cartesian<F>) {
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

impl<F, CI> DrawableColourShape<F, CI> for ColourShape<F, CI>
where
    F: ColourComponent,
    CI: ColouredItem<F>,
{
    fn proximity_to(&self, point: Point<F>) -> Proximity<F> {
        let distance = (self.xy - point).hypot();
        // TODO: implement enclosed component of proximity_to()
        Proximity::NotEnclosed(distance)
    }

    fn coloured_item(&self) -> Rc<CI> {
        Rc::clone(&self.coloured_item)
    }
}

pub fn colour_attr_xy<F: ColourComponent>(
    coloured_item: Rc<impl ColourInterface<F>>,
    scalar_attribute: ScalarAttribute,
) -> Point<F> {
    if let Some(hue_angle) = coloured_item.hue_angle() {
        let radius = coloured_item.scalar_attribute(scalar_attribute);
        Point::from((hue_angle, radius))
    } else {
        Point {
            x: F::from(-1.05).unwrap(),
            y: F::ONE - F::TWO * coloured_item.value(),
        }
    }
}

pub trait XYForAttribute<F: ColourComponent>: ColourInterface<F> {
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
}

pub struct Square<F, CI>
where
    F: ColourComponent,
    CI: ColouredItem<F>,
{
    coloured_item: Rc<CI>,
    xy: Point<F>,
    outline_rgb: RGB<F>,
}

impl<F, CI> Square<F, CI>
where
    F: ColourComponent,
    CI: ColouredItem<F> + XYForAttribute<F>,
{
    //pub fn new(coloured_item: Rc<CI>, scalar_attribute: ScalarAttribute) -> Self {
    //     let xy = coloured_item.xy_for_attribute(scalar_attribute);
    //}
}
