// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Dirn, Draw, Point, TextPosn};
use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};
use normalised_angles::{DegreesConst, RadiansConst};

pub trait ColourAttributeDisplayIfce<F: ColourComponent + DegreesConst + RadiansConst> {
    const LABEL: &'static str;
    const ATTRIBUTE: ScalarAttribute;

    fn new() -> Self;

    fn set_colour(&mut self, colour: Option<impl ColourInterface<F>>);
    fn attr_value(&self) -> Option<F>;
    fn attr_value_fg_rgb(&self) -> RGB<F>;

    fn set_target_colour(&mut self, colour: Option<impl ColourInterface<F>>);
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
            let font_size = F::from(15.0).unwrap();
            drawer.set_text_colour(self.label_colour());
            drawer.draw_text(Self::LABEL, posn, font_size);
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

// VALUE
pub struct ValueCAD<F: ColourComponent + DegreesConst + RadiansConst> {
    value: Option<F>,
    target_value: Option<F>,
    value_fg_rgb: RGB<F>,
    target_value_fg_rgb: RGB<F>,
}

impl<F> ColourAttributeDisplayIfce<F> for ValueCAD<F>
where
    F: ColourComponent + DegreesConst + RadiansConst,
{
    const LABEL: &'static str = "Value";
    const ATTRIBUTE: ScalarAttribute = ScalarAttribute::Value;

    fn new() -> Self {
        Self {
            value: None,
            target_value: None,
            value_fg_rgb: RGB::BLACK,
            target_value_fg_rgb: RGB::BLACK,
        }
    }

    fn set_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            self.value = Some(colour.value());
            self.value_fg_rgb = colour.monotone_rgb().best_foreground_rgb();
        } else {
            self.value = None;
            self.value_fg_rgb = RGB::BLACK;
        }
    }

    fn attr_value(&self) -> Option<F> {
        self.value
    }

    fn attr_value_fg_rgb(&self) -> RGB<F> {
        self.value_fg_rgb
    }

    fn set_target_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            self.target_value = Some(colour.value());
            self.target_value_fg_rgb = colour.monotone_rgb().best_foreground_rgb();
        } else {
            self.target_value = None;
            self.target_value_fg_rgb = RGB::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<F> {
        self.target_value
    }

    fn attr_target_value_fg_rgb(&self) -> RGB<F> {
        self.target_value_fg_rgb
    }
}

// Warmth
pub struct WarmthCAD<F: ColourComponent + DegreesConst + RadiansConst> {
    warmth: Option<F>,
    target_warmth: Option<F>,
    warmth_fg_rgb: RGB<F>,
    target_warmth_fg_rgb: RGB<F>,
}

impl<F> ColourAttributeDisplayIfce<F> for WarmthCAD<F>
where
    F: ColourComponent + DegreesConst + RadiansConst,
{
    const LABEL: &'static str = "Warmth";
    const ATTRIBUTE: ScalarAttribute = ScalarAttribute::Warmth;

    fn new() -> Self {
        Self {
            warmth: None,
            target_warmth: None,
            warmth_fg_rgb: RGB::BLACK,
            target_warmth_fg_rgb: RGB::BLACK,
        }
    }

    fn set_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            self.warmth = Some(colour.warmth());
            self.warmth_fg_rgb = colour.monotone_rgb().best_foreground_rgb();
        } else {
            self.warmth = None;
            self.warmth_fg_rgb = RGB::BLACK;
        }
    }

    fn attr_value(&self) -> Option<F> {
        self.warmth
    }

    fn attr_value_fg_rgb(&self) -> RGB<F> {
        self.warmth_fg_rgb
    }

    fn set_target_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            self.target_warmth = Some(colour.warmth());
            self.target_warmth_fg_rgb = colour.monotone_rgb().best_foreground_rgb();
        } else {
            self.target_warmth = None;
            self.target_warmth_fg_rgb = RGB::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<F> {
        self.target_warmth
    }

    fn attr_target_value_fg_rgb(&self) -> RGB<F> {
        self.target_warmth_fg_rgb
    }

    fn colour_stops(&self) -> Vec<(RGB<F>, F)> {
        let grey = RGB::WHITE * F::HALF;
        vec![(RGB::CYAN, F::ZERO), (grey, F::HALF), (RGB::RED, F::ONE)]
    }
}
