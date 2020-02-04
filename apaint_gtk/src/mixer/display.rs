// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{collections::BTreeMap, rc::Rc};

use gtk::prelude::*;

use colour_math::{ColourInterface, ScalarAttribute};

use pw_gix::{gtkx::coloured::*, gtkx::dialog::dialog_user::TopGtkWindow, wrapper::*};

use apaint::{characteristics::CharacteristicType, mixtures::Mixture, BasicPaintIfce};

use crate::{
    attributes::ColourAttributeDisplayStack,
    colour::RGB,
    list::{ColouredItemListView, ColouredItemListViewSpec, PaintListRow},
};

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

#[derive(Default)]
pub struct MixtureDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB>,
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
