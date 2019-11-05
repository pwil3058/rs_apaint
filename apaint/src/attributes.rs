// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Dirn, Draw, Point, TextPosn};
use colour_math::{ColourComponent, ScalarAttribute, RGB};
use normalised_angles::{DegreesConst, RadiansConst};

pub trait ColourAttributeDisplayIfce<F: ColourComponent + DegreesConst + RadiansConst> {
    const LABEL: &'static str;
    const ATTRIBUTE: ScalarAttribute;

    fn new() -> Self;

    fn set_colour(&mut self, rgb: Option<RGB<F>>);
    fn attr_value(&self) -> Option<F>;
    fn attr_value_fg_rgb(&self) -> RGB<F>;

    fn set_target_colour(&mut self, rgb: Option<RGB<F>>);
    fn attr_target_value(&self) -> Option<F>;
    fn attr_target_value_fg_rgb(&self) -> RGB<F>;

    fn label_colour(&self) -> RGB<F> {
        match self.attr_value() {
            Some(_) => self.attr_value_fg_rgb(),
            None => match self.attr_target_value() {
                Some(_) => self.attr_target_value_fg_rgb(),
                None => RGB::BLACK,
            },
        }
    }

    fn colour_stops(&self) -> Vec<(RGB<F>, F)> {
        vec![(RGB::BLACK, F::ZERO), (RGB::WHITE, F::ONE)]
    }

    fn draw_attr_value_indicator(&self, drawer: &impl Draw<F>) {
        if let Some(attr_value) = self.attr_value() {
            let size = drawer.size();
            let indicator_x = size.width * attr_value;
            drawer.set_fill_colour(self.attr_value_fg_rgb());
            drawer.set_line_colour(self.attr_value_fg_rgb());
            let side: F = F::from(8.0).unwrap();
            drawer.draw_isosceles((indicator_x, F::ONE).into(), Dirn::Up, side, true);
            drawer.draw_isosceles(
                (indicator_x, size.height - F::ONE).into(),
                Dirn::Down,
                side,
                true,
            );
        }
    }

    fn draw_target_attr_value_indicator(&self, drawer: &impl Draw<F>) {
        if let Some(attr_value) = self.attr_value() {
            let size = drawer.size();
            let indicator_x = size.width * attr_value;
            drawer.set_line_width(F::TWO);
            drawer.set_line_colour(self.attr_value_fg_rgb());
            drawer.draw_line(&[
                (indicator_x, F::ONE).into(),
                (indicator_x, size.height - F::ONE).into(),
            ]);
        }
    }

    fn draw_label(&self, drawer: &impl Draw<F>) {
        if Self::LABEL.len() > 0 {
            let posn = TextPosn::Centre(drawer.size().centre());
            let rgb = self.label_colour();
            let font_size = F::from(15.0).unwrap();
            drawer.draw_text(Self::LABEL, posn, font_size, rgb);
        }
    }

    fn draw_background(&self, drawer: &impl Draw<F>) {
        let posn = Point::<F>::default();
        let size = drawer.size();
        drawer.paint_linear_gradient(posn, size, &self.colour_stops());
    }

    fn draw_all(&self, drawer: &impl Draw<F>) {
        self.draw_background(drawer);
        self.draw_target_attr_value_indicator(drawer);
        self.draw_attr_value_indicator(drawer);
        self.draw_label(drawer);
    }
}
