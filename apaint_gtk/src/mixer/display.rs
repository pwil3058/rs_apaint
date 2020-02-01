// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{collections::BTreeMap, rc::Rc};

use gtk::prelude::*;

use colour_math::{ColourInterface, ScalarAttribute};

use pw_gix::{gtkx::coloured::*, gtkx::dialog::dialog_user::TopGtkWindow, wrapper::*};

use apaint::{characteristics::CharacteristicType, mixtures::Mixture, BasicPaintIfce};

use crate::{attributes::ColourAttributeDisplayStack, colour::RGB};

#[derive(PWO)]
pub struct MixtureDisplay {
    vbox: gtk::Box,
    mixture: Rc<Mixture<f64>>,
    target_label: gtk::Label,
    cads: ColourAttributeDisplayStack,
}

impl MixtureDisplay {
    pub fn set_target(&self, new_target: Option<&RGB>) {
        if let Some(rgb) = new_target {
            self.target_label.set_label("Current Target");
            self.target_label.set_widget_colour_rgb(*rgb);
            self.cads.set_target_colour(Some(rgb));
        } else {
            self.target_label.set_label("");
            self.target_label.set_widget_colour_rgb(self.mixture.rgb());
            self.cads.set_target_colour(Option::<&RGB>::None);
        };
    }

    pub fn mixture(&self) -> &Rc<Mixture<f64>> {
        &self.mixture
    }
}

pub struct MixtureDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB>,
}

impl MixtureDisplayBuilder {
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

    pub fn target_rgb(&mut self, target_rgb: Option<&RGB>) -> &mut Self {
        self.target_rgb = if let Some(target_rgb) = target_rgb {
            Some(*target_rgb)
        } else {
            None
        };
        self
    }

    pub fn build(&self, mixture: &Rc<Mixture<f64>>) -> MixtureDisplay {
        let rgb = mixture.rgb();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let label = gtk::LabelBuilder::new().label(mixture.id()).build();
        label.set_widget_colour_rgb(rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(mixture.name().unwrap_or(""))
            .build();
        label.set_widget_colour_rgb(rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(mixture.notes().unwrap_or(""))
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

        if let Some(targeted_rgb) = mixture.targeted_rgb() {
            let label = gtk::LabelBuilder::new().label("Matched Colour").build();
            label.set_widget_colour_rgb(*targeted_rgb);
            vbox.pack_start(&label, true, true, 0);
        }

        vbox.pack_start(&cads.pwo(), true, true, 0);

        for characteristic_type in self.characteristics.iter() {
            let value = mixture.characteristic(*characteristic_type).full();
            let label = gtk::LabelBuilder::new().label(&value).build();
            label.set_widget_colour_rgb(rgb);
            vbox.pack_start(&label, false, false, 0);
        }
        vbox.show_all();

        MixtureDisplay {
            vbox,
            mixture: Rc::clone(mixture),
            target_label,
            cads,
        }
    }
}

struct MixtureDisplayDialog {
    pub dialog: gtk::Dialog,
    pub display: MixtureDisplay,
}

pub struct MixtureDisplayDialogManager<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(String, gtk::ResponseType)>,
    mixture_display_builder: MixtureDisplayBuilder,
    dialogs: BTreeMap<Rc<Mixture<f64>>, MixtureDisplayDialog>,
}

impl<W: TopGtkWindow> MixtureDisplayDialogManager<W> {
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

    pub fn display_mixture(&mut self, mixture: &Rc<Mixture<f64>>) {
        if !self.dialogs.contains_key(mixture) {
            let dialog = self.new_dialog();
            let display = self.mixture_display_builder.build(mixture);
            dialog
                .get_content_area()
                .pack_start(&display.pwo(), true, true, 0);
            let pdd = MixtureDisplayDialog { dialog, display };
            self.dialogs.insert(Rc::clone(mixture), pdd);
        };
        let pdd = self.dialogs.get(mixture).expect("we just pit it there");
        pdd.dialog.present();
    }

    pub fn set_target_rgb(&mut self, rgb: Option<&RGB>) {
        self.mixture_display_builder.target_rgb(rgb);
        for pdd in self.dialogs.values() {
            pdd.display.set_target(rgb);
        }
    }
}

pub struct MixtureDisplayDialogManagerBuilder<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(String, gtk::ResponseType)>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB>,
}

impl<W: TopGtkWindow + Clone> MixtureDisplayDialogManagerBuilder<W> {
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

    pub fn build(&self) -> MixtureDisplayDialogManager<W> {
        let mut mixture_display_builder = MixtureDisplayBuilder::new();
        mixture_display_builder
            .attributes(&self.attributes)
            .characteristics(&self.characteristics);
        if let Some(target_rgb) = self.target_rgb {
            mixture_display_builder.target_rgb(Some(&target_rgb));
        }
        MixtureDisplayDialogManager {
            caller: self.caller.clone(),
            buttons: self.buttons.clone(),
            mixture_display_builder,
            dialogs: BTreeMap::new(),
        }
    }
}
