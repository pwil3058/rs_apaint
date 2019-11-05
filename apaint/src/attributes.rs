// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::Draw;
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

    fn draw_attr_value_indicator(&self, drawer: impl Draw<F>) {
        if let Some(attr_value) = self.attr_value() {
            let size = drawer.size();
            let _indicator_x = size.width * attr_value;
        }
    }
}
