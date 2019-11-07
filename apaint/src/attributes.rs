// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::drawing::{Dirn, Draw, Point, TextPosn};
use colour_math::{ColourComponent, ColourInterface, Hue, RGB};
use normalised_angles::{Degrees, DegreesConst, RadiansConst};

pub trait ColourAttributeDisplayIfce<F: ColourComponent + DegreesConst + RadiansConst> {
    const LABEL: &'static str;

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
            let base: F = F::from(8.0).unwrap();
            let height: F = F::from(6.0).unwrap();
            drawer.draw_isosceles(
                (indicator_x, F::HALF * height).into(),
                Dirn::Up,
                base,
                height,
                true,
            );
            drawer.draw_isosceles(
                (indicator_x, size.height - F::HALF * height).into(),
                Dirn::Down,
                base,
                height,
                true,
            );
        }
    }

    fn draw_target_attr_value_indicator(&self, drawer: &impl Draw<F>) {
        if let Some(attr_value) = self.attr_target_value() {
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

    fn label_colour(&self) -> RGB<F> {
        RGB::WHITE
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

    fn label_colour(&self) -> RGB<F> {
        RGB::WHITE
    }

    fn colour_stops(&self) -> Vec<(RGB<F>, F)> {
        let grey = RGB::WHITE * F::HALF;
        vec![(RGB::CYAN, F::ZERO), (grey, F::HALF), (RGB::RED, F::ONE)]
    }
}

// HUE
pub struct HueCAD<F: ColourComponent + DegreesConst + RadiansConst> {
    hue: Option<Hue<F>>,
    target_hue: Option<Hue<F>>,
    hue_value: Option<F>,
    hue_fg_rgb: RGB<F>,
    target_hue_fg_rgb: RGB<F>,
    colour_stops: Vec<(RGB<F>, F)>,
}

impl<F: ColourComponent + DegreesConst + RadiansConst> HueCAD<F> {
    fn set_colour_stops_for_hue(&mut self, hue: Hue<F>) {
        let mut stops = vec![];
        let mut hue = hue + Degrees::DEG_180;
        let delta = Degrees::DEG_30;
        for i in 0..13 {
            let offset = F::from_usize(i).unwrap() / F::from(12.0).unwrap();
            let rgb = hue.max_chroma_rgb();
            stops.push((rgb, offset));
            hue = hue - delta;
        }
        self.colour_stops = stops
    }

    fn set_colour_stops(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            if let Some(hue) = colour.hue() {
                self.set_colour_stops_for_hue(hue);
            } else {
                let grey = colour.rgb();
                self.colour_stops = vec![(grey, F::ZERO), (grey, F::ONE)];
            }
        } else {
            self.set_default_colour_stops();
        }
    }

    fn set_default_colour_stops(&mut self) {
        let grey = RGB::WHITE * F::HALF;
        self.colour_stops = vec![(grey, F::ZERO), (grey, F::ONE)];
    }

    fn set_defaults_for_no_hue(&mut self) {
        self.hue = None;
        self.hue_value = None;
        if let Some(target_hue) = self.target_hue {
            self.set_colour_stops_for_hue(target_hue);
        } else {
            self.set_default_colour_stops();
        }
    }

    fn set_defaults_for_no_target(&mut self) {
        self.target_hue = None;
        if let Some(hue) = self.hue {
            self.set_colour_stops_for_hue(hue);
            self.hue_value = Some(F::HALF);
        } else {
            self.set_default_colour_stops();
        }
    }

    fn calc_hue_value(hue: Hue<F>, target_hue: Hue<F>) -> F {
        F::HALF - (target_hue - hue).degrees() / Degrees::DEG_360.degrees()
    }
}

impl<F: ColourComponent + DegreesConst + RadiansConst> ColourAttributeDisplayIfce<F> for HueCAD<F> {
    const LABEL: &'static str = "Hue";

    fn new() -> Self {
        let grey = RGB::WHITE * F::HALF;
        Self {
            hue: None,
            target_hue: None,
            hue_value: None,
            hue_fg_rgb: RGB::BLACK,
            target_hue_fg_rgb: RGB::BLACK,
            colour_stops: vec![(grey, F::ZERO), (grey, F::ONE)],
        }
    }

    fn set_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            if let Some(hue) = colour.hue() {
                self.hue = Some(hue);
                self.hue_fg_rgb = hue.max_chroma_rgb().best_foreground_rgb();
                if let Some(target_hue) = self.target_hue {
                    self.hue_value = Some(Self::calc_hue_value(hue, target_hue));
                } else {
                    self.set_colour_stops(Some(colour));
                    self.hue_value = Some(F::HALF);
                }
            } else {
                self.set_defaults_for_no_hue()
            }
        } else {
            self.set_defaults_for_no_hue()
        }
    }

    fn attr_value(&self) -> Option<F> {
        self.hue_value
    }

    fn attr_value_fg_rgb(&self) -> RGB<F> {
        self.hue_fg_rgb
    }

    fn set_target_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            if let Some(target_hue) = colour.hue() {
                self.target_hue = Some(target_hue);
                self.target_hue_fg_rgb = target_hue.max_chroma_rgb().best_foreground_rgb();
                self.set_colour_stops_for_hue(target_hue);
                if let Some(hue) = self.hue {
                    self.hue_value = Some(Self::calc_hue_value(hue, target_hue));
                }
            } else {
                self.set_defaults_for_no_target();
            }
        } else {
            self.set_defaults_for_no_target();
        }
    }

    fn attr_target_value(&self) -> Option<F> {
        if self.target_hue.is_some() {
            Some(F::HALF)
        } else {
            None
        }
    }

    fn attr_target_value_fg_rgb(&self) -> RGB<F> {
        self.target_hue_fg_rgb
    }

    fn colour_stops(&self) -> Vec<(RGB<F>, F)> {
        self.colour_stops.clone()
    }
}

// Chroma
pub struct ChromaCAD<F: ColourComponent + DegreesConst + RadiansConst> {
    chroma: Option<F>,
    target_chroma: Option<F>,
    chroma_fg_rgb: RGB<F>,
    target_chroma_fg_rgb: RGB<F>,
    colour_stops: Vec<(RGB<F>, F)>,
}

impl<F: ColourComponent + DegreesConst + RadiansConst> ChromaCAD<F> {
    fn set_colour_stops(&mut self, colour: Option<impl ColourInterface<F>>) {
        self.colour_stops = if let Some(colour) = colour {
            if colour.is_grey() {
                let grey = colour.rgb();
                vec![(grey, F::ZERO), (grey, F::ONE)]
            } else {
                let start_rgb = colour.monotone_rgb();
                let end_rgb = colour.max_chroma_rgb();
                vec![(start_rgb, F::ZERO), (end_rgb, F::ONE)]
            }
        } else {
            Self::default_colour_stops()
        }
    }

    fn default_colour_stops() -> Vec<(RGB<F>, F)> {
        let grey = RGB::WHITE * F::HALF;
        vec![(grey, F::ZERO), (grey, F::ONE)]
    }
}

impl<F> ColourAttributeDisplayIfce<F> for ChromaCAD<F>
where
    F: ColourComponent + DegreesConst + RadiansConst,
{
    const LABEL: &'static str = "Chroma";

    fn new() -> Self {
        let grey = RGB::WHITE * F::HALF;
        Self {
            chroma: None,
            target_chroma: None,
            chroma_fg_rgb: RGB::BLACK,
            target_chroma_fg_rgb: RGB::BLACK,
            colour_stops: vec![(grey, F::ZERO), (grey, F::ONE)],
        }
    }

    fn set_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            self.chroma = Some(colour.chroma());
            self.chroma_fg_rgb = colour.best_foreground_rgb();
            if let Some(target_chroma) = self.target_chroma {
                if target_chroma == F::ZERO {
                    self.set_colour_stops(Some(colour));
                }
            } else {
                self.set_colour_stops(Some(colour));
            }
        } else {
            self.chroma = None;
            self.chroma_fg_rgb = RGB::BLACK;
            if self.target_chroma.is_none() {
                self.colour_stops = Self::default_colour_stops()
            }
        }
    }

    fn attr_value(&self) -> Option<F> {
        self.chroma
    }

    fn attr_value_fg_rgb(&self) -> RGB<F> {
        self.chroma_fg_rgb
    }

    fn set_target_colour(&mut self, colour: Option<impl ColourInterface<F>>) {
        if let Some(colour) = colour {
            self.target_chroma = Some(colour.chroma());
            self.target_chroma_fg_rgb = colour.monotone_rgb().best_foreground_rgb();
            if colour.is_grey() {
                if let Some(chroma) = self.chroma {
                    if chroma == F::ZERO {
                        self.set_colour_stops(Some(colour));
                    }
                } else {
                    self.set_colour_stops(Some(colour));
                }
            } else {
                self.set_colour_stops(Some(colour));
            }
        } else {
            self.target_chroma = None;
            self.target_chroma_fg_rgb = RGB::BLACK;
        }
    }

    fn attr_target_value(&self) -> Option<F> {
        self.target_chroma
    }

    fn attr_target_value_fg_rgb(&self) -> RGB<F> {
        self.target_chroma_fg_rgb
    }

    fn label_colour(&self) -> RGB<F> {
        RGB::WHITE
    }

    fn colour_stops(&self) -> Vec<(RGB<F>, F)> {
        self.colour_stops.clone()
    }
}
