// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use colour_math::{ColourInterface, ScalarAttribute};

use pw_gix::{gtkx::coloured::*, wrapper::*};

use apaint::{characteristics::CharacteristicType, series::SeriesPaint, BasicPaintIfce};

use crate::{attributes::ColourAttributeDisplayStack, colour::RGB};

#[derive(PWO)]
pub struct PaintDisplay {
    vbox: gtk::Box,
    paint: Rc<SeriesPaint<f64>>,
    target_label: gtk::Label,
    cads: ColourAttributeDisplayStack,
}

impl PaintDisplay {
    pub fn set_target(&self, new_target: Option<&RGB>) {
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

    pub fn paint(&self) -> &Rc<SeriesPaint<f64>> {
        &self.paint
    }
}

pub struct PaintDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB>,
}

impl PaintDisplayBuilder {
    pub fn new() -> Self {
        Self {
            attributes: vec![],
            characteristics: vec![],
            target_rgb: None,
        }
    }

    pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
        self.attributes = attributes.to_vec();
        self
    }

    pub fn characteristics(&mut self, characteristics: &[CharacteristicType]) -> &mut Self {
        self.characteristics = characteristics.to_vec();
        self
    }

    pub fn target_rgb(&mut self, target_rgb: &RGB) -> &mut Self {
        self.target_rgb = Some(*target_rgb);
        self
    }

    pub fn build(&self, paint: &Rc<SeriesPaint<f64>>) -> PaintDisplay {
        let rgb = paint.rgb();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let label = gtk::LabelBuilder::new().label(paint.id()).build();
        label.set_widget_colour_rgb(rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.name().unwrap_or(""))
            .build();
        label.set_widget_colour_rgb(rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.notes().unwrap_or(""))
            .build();
        label.set_widget_colour_rgb(rgb);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.series_name())
            .build();
        label.set_widget_colour_rgb(rgb);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.proprietor())
            .build();
        label.set_widget_colour_rgb(rgb);
        vbox.pack_start(&label, false, false, 0);

        let cads = ColourAttributeDisplayStack::new(&self.attributes);
        cads.set_colour(Some(&rgb));
        let target_label = if let Some(target_rgb) = self.target_rgb {
            let label = gtk::LabelBuilder::new().label("Target").build();
            label.set_widget_colour_rgb(target_rgb);
            cads.set_target_colour(Some(&target_rgb));
            label
        } else {
            let label = gtk::LabelBuilder::new().build();
            label.set_widget_colour_rgb(rgb);
            label
        };
        vbox.pack_start(&target_label, true, true, 0);
        vbox.pack_start(&cads.pwo(), true, true, 0);

        for characteristic_type in self.characteristics.iter() {
            let value = paint.characteristic(*characteristic_type).full();
            let label = gtk::LabelBuilder::new().label(&value).build();
            label.set_widget_colour_rgb(rgb);
            vbox.pack_start(&label, false, false, 0);
        }

        PaintDisplay {
            vbox,
            paint: Rc::clone(paint),
            target_label,
            cads,
        }
    }
}
