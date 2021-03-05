// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{collections::BTreeMap, rc::Rc};

use pw_gix::{
    glib,
    gtk::{self, prelude::*},
    gtkx::dialog::dialog_user::TopGtkWindow,
    wrapper::*,
};

use colour_math::{ColourBasics, ScalarAttribute};
use colour_math_gtk::attributes::ColourAttributeDisplayStackBuilder;

#[cfg(feature = "targeted_mixtures")]
use colour_math_gtk::{attributes::ColourAttributeDisplayStack, colour::*};

use apaint::{characteristics::CharacteristicType, mixtures::Mixture, BasicPaintIfce};

use crate::{
    colour::{Colourable, HCV},
    list::{ColouredItemListView, ColouredItemListViewSpec, PaintListRow},
};

#[derive(PWO)]
pub struct MixtureDisplay {
    vbox: gtk::Box,
    mixture: Rc<Mixture>,
    #[cfg(feature = "targeted_mixtures")]
    target_label: gtk::Label,
    #[cfg(feature = "targeted_mixtures")]
    cads: Rc<ColourAttributeDisplayStack>,
}

impl MixtureDisplay {
    #[cfg(feature = "targeted_mixtures")]
    pub fn set_target(&self, new_target: Option<&impl GdkColour>) {
        if let Some(colour) = new_target {
            self.target_label.set_label("Current Target");
            self.target_label.set_widget_colour(colour);
            self.cads.set_target_colour(Some(colour));
        } else {
            self.target_label.set_label("");
            self.target_label.set_widget_colour(&self.mixture.hcv());
            self.cads.set_target_colour(Option::<&HCV>::None);
        };
    }

    pub fn mixture(&self) -> &Rc<Mixture> {
        &self.mixture
    }
}

#[derive(Default)]
pub struct MixtureDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    #[cfg(feature = "targeted_mixtures")]
    target_colour: Option<HCV>,
    list_spec: ComponentsListViewSpec,
}

impl MixtureDisplayBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
        self.attributes = attributes.to_vec();
        self.list_spec = ComponentsListViewSpec::new(&self.attributes, &self.characteristics);
        self
    }

    pub fn characteristics(&mut self, characteristics: &[CharacteristicType]) -> &mut Self {
        self.characteristics = characteristics.to_vec();
        self.list_spec = ComponentsListViewSpec::new(&self.attributes, &self.characteristics);
        self
    }

    #[cfg(feature = "targeted_mixtures")]
    pub fn target_colour(&mut self, target_colour: Option<&impl ColourBasics>) -> &mut Self {
        self.target_colour = if let Some(target_colour) = target_colour {
            Some(target_colour.hcv())
        } else {
            None
        };
        self
    }

    pub fn build(&self, mixture: &Rc<Mixture>) -> MixtureDisplay {
        let colour = mixture.hcv();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let label = gtk::LabelBuilder::new().label(mixture.id()).build();
        label.set_widget_colour(&colour);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(mixture.name().unwrap_or(""))
            .build();
        label.set_widget_colour(&colour);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(mixture.notes().unwrap_or(""))
            .build();
        label.set_widget_colour(&colour);
        vbox.pack_start(&label, false, false, 0);

        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(&self.attributes)
            .build();
        cads.set_colour(Some(&colour));

        #[cfg(feature = "targeted_mixtures")]
        let target_label = if let Some(target_colour) = self.target_colour {
            let label = gtk::LabelBuilder::new().label("Target").build();
            label.set_widget_colour(&target_colour);
            cads.set_target_colour(Some(&target_colour));
            label
        } else {
            let label = gtk::LabelBuilder::new().build();
            label.set_widget_colour(&colour);
            label
        };
        #[cfg(feature = "targeted_mixtures")]
        vbox.pack_start(&target_label, true, true, 0);

        #[cfg(feature = "targeted_mixtures")]
        if let Some(targeted_colour) = mixture.targeted_colour() {
            let label = gtk::LabelBuilder::new().label("Matched Colour").build();
            label.set_widget_colour(&targeted_colour);
            vbox.pack_start(&label, true, true, 0);
        }

        vbox.pack_start(&cads.pwo(), true, true, 0);

        for characteristic_type in self.characteristics.iter() {
            let value = mixture.characteristic(*characteristic_type).full();
            let label = gtk::LabelBuilder::new().label(&value).build();
            label.set_widget_colour(&colour);
            vbox.pack_start(&label, false, false, 0);
        }

        let list_view = ColouredItemListView::new(&self.list_spec, &[]);
        vbox.pack_start(&list_view.pwo(), false, false, 0);
        for (paint, parts) in mixture.components() {
            let mut row = paint.row(&self.attributes, &self.characteristics);
            let value: glib::Value = (*parts as u64).to_value();
            row.insert(7, value);
            list_view.add_row(&row);
        }

        vbox.show_all();

        MixtureDisplay {
            vbox,
            mixture: Rc::clone(mixture),
            #[cfg(feature = "targeted_mixtures")]
            target_label,
            #[cfg(feature = "targeted_mixtures")]
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
    buttons: Vec<(&'static str, Option<&'static str>, u16)>,
    mixture_display_builder: MixtureDisplayBuilder,
    dialogs: BTreeMap<Rc<Mixture>, MixtureDisplayDialog>,
}

impl<W: TopGtkWindow> MixtureDisplayDialogManager<W> {
    fn new_dialog(&self) -> gtk::Dialog {
        let dialog = gtk::DialogBuilder::new().build();
        if let Some(parent) = self.caller.get_toplevel_gtk_window() {
            dialog.set_transient_for(Some(&parent));
        }
        for (label, tooltip_text, response) in self.buttons.iter() {
            dialog
                .add_button(label, gtk::ResponseType::Other(*response))
                .set_tooltip_text(*tooltip_text);
        }
        // TODO: think about removal from map as an optional action to hiding
        dialog.connect_delete_event(|d, _| {
            d.hide_on_delete();
            gtk::Inhibit(true)
        });
        dialog
    }

    pub fn display_mixture(&mut self, mixture: &Rc<Mixture>) {
        if !self.dialogs.contains_key(mixture) {
            let dialog = self.new_dialog();
            let display = self.mixture_display_builder.build(mixture);
            dialog
                .get_content_area()
                .pack_start(&display.pwo(), true, true, 0);
            let pdd = MixtureDisplayDialog { dialog, display };
            self.dialogs.insert(Rc::clone(mixture), pdd);
        };
        let pdd = self.dialogs.get(mixture).expect("we just put it there");
        pdd.dialog.present();
    }

    #[cfg(feature = "targeted_mixtures")]
    pub fn set_target_colour(&mut self, rgb: Option<&impl GdkColour>) {
        self.mixture_display_builder.target_colour(rgb);
        for pdd in self.dialogs.values() {
            pdd.display.set_target(rgb);
        }
    }
}

pub struct MixtureDisplayDialogManagerBuilder<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(&'static str, Option<&'static str>, u16)>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_colour: Option<HCV>,
}

impl<W: TopGtkWindow + Clone> MixtureDisplayDialogManagerBuilder<W> {
    pub fn new(caller: &W) -> Self {
        Self {
            caller: caller.clone(),
            buttons: vec![],
            attributes: vec![],
            characteristics: vec![],
            target_colour: None,
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

    pub fn buttons(&mut self, buttons: &[(&'static str, Option<&'static str>, u16)]) -> &mut Self {
        self.buttons = buttons.to_vec();
        self
    }

    pub fn target_colour(&mut self, target_colour: &impl ColourBasics) {
        self.target_colour = Some(target_colour.hcv());
    }

    pub fn build(&self) -> MixtureDisplayDialogManager<W> {
        let mut mixture_display_builder = MixtureDisplayBuilder::new();
        mixture_display_builder
            .attributes(&self.attributes)
            .characteristics(&self.characteristics);
        #[cfg(feature = "targeted_mixtures")]
        if let Some(target_colour) = self.target_colour {
            mixture_display_builder.target_colour(Some(&target_colour));
        }
        MixtureDisplayDialogManager {
            caller: self.caller.clone(),
            buttons: self.buttons.clone(),
            mixture_display_builder,
            dialogs: BTreeMap::new(),
        }
    }
}

#[derive(Default)]
pub struct ComponentsListViewSpec {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
}

impl ComponentsListViewSpec {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Self {
        Self {
            attributes: attributes.to_vec(),
            characteristics: characteristics.to_vec(),
        }
    }
}

impl ColouredItemListViewSpec for ComponentsListViewSpec {
    fn column_types(&self) -> Vec<glib::Type> {
        let mut column_types = vec![
            glib::Type::String,
            glib::Type::String,
            glib::Type::String,
            glib::Type::String,
            glib::Type::String,
            glib::Type::String,
            f64::static_type(),
            u64::static_type(),
        ];
        for _ in 0..self.attributes.len() * 3 + self.characteristics.len() {
            column_types.push(glib::Type::String);
        }
        column_types
    }

    fn columns(&self) -> Vec<gtk::TreeViewColumn> {
        let mut cols = vec![];

        let col = gtk::TreeViewColumnBuilder::new()
            .title("Parts")
            .resizable(false)
            .sort_column_id(7)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "text", 7);
        //col.add_attribute(&cell, "background", 1);
        //col.add_attribute(&cell, "foreground", 2);
        cols.push(col);

        let col = gtk::TreeViewColumnBuilder::new()
            .title("Id")
            .resizable(false)
            .sort_column_id(0)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "text", 0);
        col.add_attribute(&cell, "background", 1);
        col.add_attribute(&cell, "foreground", 2);
        cols.push(col);

        let col = gtk::TreeViewColumnBuilder::new()
            .title("Name")
            .resizable(true)
            .sort_column_id(3)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "text", 3);
        col.add_attribute(&cell, "background", 1);
        col.add_attribute(&cell, "foreground", 2);
        cols.push(col);

        let col = gtk::TreeViewColumnBuilder::new()
            .title("Notes")
            .resizable(true)
            .sort_column_id(4)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "text", 4);
        col.add_attribute(&cell, "background", 1);
        col.add_attribute(&cell, "foreground", 2);
        cols.push(col);

        let col = gtk::TreeViewColumnBuilder::new()
            .title("Hue")
            .sort_column_id(6)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "background", 5);
        cols.push(col);

        let mut index = 8;
        for attr in self.attributes.iter() {
            let col = gtk::TreeViewColumnBuilder::new()
                .title(&attr.to_string())
                .sort_column_id(index)
                .sort_indicator(true)
                .build();
            let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
            col.pack_start(&cell, false);
            col.add_attribute(&cell, "text", index);
            col.add_attribute(&cell, "background", index + 1);
            col.add_attribute(&cell, "foreground", index + 2);
            cols.push(col);
            index += 3;
        }

        for characteristic in self.characteristics.iter() {
            let col = gtk::TreeViewColumnBuilder::new()
                .title(&characteristic.list_header_name())
                .sort_column_id(index)
                .sort_indicator(true)
                .build();
            let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
            col.pack_start(&cell, false);
            col.add_attribute(&cell, "text", index);
            col.add_attribute(&cell, "background", 1);
            col.add_attribute(&cell, "foreground", 2);
            cols.push(col);
            index += 1;
        }

        cols
    }
}
