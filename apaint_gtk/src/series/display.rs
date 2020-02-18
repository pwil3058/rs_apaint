// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{collections::BTreeMap, rc::Rc};

use gtk::prelude::*;

use colour_math::{ColourInterface, ScalarAttribute};
use colour_math_gtk::attributes::{
    ColourAttributeDisplayStack, ColourAttributeDisplayStackBuilder,
};

use pw_gix::{gtkx::dialog::dialog_user::TopGtkWindow, wrapper::*};

use apaint::{characteristics::CharacteristicType, series::SeriesPaint, BasicPaintIfce};

use crate::colour::{Colourable, RGB};

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
            self.target_label.set_widget_colour_rgb(rgb);
            self.cads.set_target_colour(Some(rgb));
        } else {
            self.target_label.set_label("");
            self.target_label.set_widget_colour_rgb(&self.paint.rgb());
            self.cads.set_target_colour(Option::<&RGB>::None);
        };
    }

    pub fn paint(&self) -> &Rc<SeriesPaint<f64>> {
        &self.paint
    }
}

#[derive(Default)]
pub struct PaintDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB>,
}

impl PaintDisplayBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
        self.attributes = attributes.to_vec();
        self
    }

    pub fn characteristics(&mut self, characteristics: &[CharacteristicType]) -> &mut Self {
        self.characteristics = characteristics.to_vec();
        self
    }

    pub fn target_rgb(&mut self, target_rgb: Option<&RGB>) -> &mut Self {
        self.target_rgb = if let Some(target_rgb) = target_rgb {
            Some(*target_rgb)
        } else {
            None
        };
        self
    }

    pub fn build(&self, paint: &Rc<SeriesPaint<f64>>) -> PaintDisplay {
        let rgb = paint.rgb();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let label = gtk::LabelBuilder::new().label(paint.id()).build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.name().unwrap_or(""))
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.notes().unwrap_or(""))
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.series_name())
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.proprietor())
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(&self.attributes)
            .build();
        cads.set_colour(Some(&rgb));
        let target_label = if let Some(target_rgb) = self.target_rgb {
            let label = gtk::LabelBuilder::new().label("Target").build();
            label.set_widget_colour_rgb(&target_rgb);
            cads.set_target_colour(Some(&target_rgb));
            label
        } else {
            let label = gtk::LabelBuilder::new().build();
            label.set_widget_colour_rgb(&rgb);
            label
        };
        vbox.pack_start(&target_label, true, true, 0);
        vbox.pack_start(&cads.pwo(), true, true, 0);

        for characteristic_type in self.characteristics.iter() {
            let value = paint.characteristic(*characteristic_type).full();
            let label = gtk::LabelBuilder::new().label(&value).build();
            label.set_widget_colour_rgb(&rgb);
            vbox.pack_start(&label, false, false, 0);
        }
        vbox.show_all();

        PaintDisplay {
            vbox,
            paint: Rc::clone(paint),
            target_label,
            cads,
        }
    }
}

struct PaintDisplayDialog {
    pub dialog: gtk::Dialog,
    pub display: PaintDisplay,
}

pub struct PaintDisplayDialogManager<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(String, gtk::ResponseType)>,
    paint_display_builder: PaintDisplayBuilder,
    dialogs: BTreeMap<Rc<SeriesPaint<f64>>, PaintDisplayDialog>,
}

impl<W: TopGtkWindow> PaintDisplayDialogManager<W> {
    fn new_dialog(&self) -> gtk::Dialog {
        let dialog = gtk::DialogBuilder::new().build();
        if let Some(parent) = self.caller.get_toplevel_gtk_window() {
            dialog.set_transient_for(Some(&parent));
        }
        for (label, response) in self.buttons.iter() {
            dialog.add_button(label, *response);
        }
        // TODO: think about removal from map as an optional action to hiding
        dialog.connect_delete_event(|d, _| {
            d.hide_on_delete();
            gtk::Inhibit(true)
        });
        dialog
    }

    pub fn display_paint(&mut self, paint: &Rc<SeriesPaint<f64>>) {
        if !self.dialogs.contains_key(paint) {
            let dialog = self.new_dialog();
            let display = self.paint_display_builder.build(paint);
            dialog
                .get_content_area()
                .pack_start(&display.pwo(), true, true, 0);
            let pdd = PaintDisplayDialog { dialog, display };
            self.dialogs.insert(Rc::clone(paint), pdd);
        };
        let pdd = self.dialogs.get(paint).expect("we just pit it there");
        pdd.dialog.present();
    }

    pub fn set_target_rgb(&mut self, rgb: Option<&RGB>) {
        self.paint_display_builder.target_rgb(rgb);
        for pdd in self.dialogs.values() {
            pdd.display.set_target(rgb);
        }
    }
}

pub struct PaintDisplayDialogManagerBuilder<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(String, gtk::ResponseType)>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB>,
}

impl<W: TopGtkWindow + Clone> PaintDisplayDialogManagerBuilder<W> {
    pub fn new(caller: &W) -> Self {
        Self {
            caller: caller.clone(),
            buttons: vec![],
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

    pub fn build(&self) -> PaintDisplayDialogManager<W> {
        let mut paint_display_builder = PaintDisplayBuilder::new();
        paint_display_builder
            .attributes(&self.attributes)
            .characteristics(&self.characteristics);
        if let Some(target_rgb) = self.target_rgb {
            paint_display_builder.target_rgb(Some(&target_rgb));
        }
        PaintDisplayDialogManager {
            caller: self.caller.clone(),
            buttons: self.buttons.clone(),
            paint_display_builder,
            dialogs: BTreeMap::new(),
        }
    }
}
