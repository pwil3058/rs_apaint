// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use colour_math::{ColourInterface, ScalarAttribute};

use pw_gix::gtkx::coloured::*;

use apaint::{characteristics::CharacteristicType, series::SeriesPaint};

use crate::{
    attributes::ColourAttributeDisplayStack,
    colour::{Colour, RGB},
};

pub struct PaintDisplay {
    vbox: gtk::Box,
    paint: Rc<SeriesPaint<f64>>,
    target_label: gtk::Label,
    cads: ColourAttributeDisplayStack,
}

impl PaintDisplay {
    fn set_target(&self, new_target: Option<&RGB>) {
        if let Some(rgb) = new_target {
            self.target_label.set_label("Current Target");
            self.target_label.set_widget_colour_rgb(*rgb);
            self.cads.set_target_colour(Some(rgb));
        } else {
            self.target_label.set_label("");
            self.target_label.set_widget_colour_rgb(self.paint.rgb());
            self.cads.set_target_colour(Option::<&RGB>::None);
        };
    }
}

pub struct PaintDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB>,
}
