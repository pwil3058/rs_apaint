// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gtk::prelude::*;

use apaint_gtk_boilerplate::PWO;
use pw_gix::{
    gtkx::{list_store::ListRowOps, menu::ManagedMenu},
    sav_state::{MaskedCondns, WidgetStatesControlled},
    wrapper::PackableWidgetObject,
};

use apaint::characteristics::CharacteristicType;
use apaint::BasicPaintIfce;
use pw_gix::gtkx::list_store::TreeModelRowOps;

use crate::colour::{ColourInterface, ScalarAttribute};
use crate::managed_menu::MenuItemSpec;
use apaint::series::SeriesPaint;
use apaint::spec::BasicPaintSpec;

#[derive(PWO)]
pub struct ColouredItemListView {
    view: gtk::TreeView,
    list_store: gtk::ListStore,
    selected_id: RefCell<Option<String>>,
    popup_menu: ManagedMenu,
    callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(&str)>>>>,
}

impl ColouredItemListView {
    pub fn new(
        column_types: &[gtk::Type],
        columns: &[gtk::TreeViewColumn],
        menu_items: &[MenuItemSpec],
    ) -> Rc<Self> {
        let list_store = gtk::ListStore::new(column_types);
        let view = gtk::TreeViewBuilder::new().headers_visible(true).build();
        view.set_model(Some(&list_store));
        view.get_selection().set_mode(gtk::SelectionMode::None);

        for col in columns.iter() {
            view.append_column(col);
        }

        let rgb_l_v = Rc::new(Self {
            view,
            list_store,
            selected_id: RefCell::new(None),
            popup_menu: ManagedMenu::new(WidgetStatesControlled::Sensitivity, None, None, &[]),
            callbacks: RefCell::new(HashMap::new()),
        });

        for spec in menu_items.iter() {
            let rgb_l_v_c = Rc::clone(&rgb_l_v);
            let name_c = spec.name().to_string();
            rgb_l_v
                .popup_menu
                .append_item(
                    spec.name(),
                    spec.label(),
                    spec.image(),
                    spec.tooltip(),
                    spec.condns(),
                )
                .connect_activate(move |_| rgb_l_v_c.menu_item_selected(&name_c));
            rgb_l_v
                .callbacks
                .borrow_mut()
                .insert(spec.name().to_string(), vec![]);
        }

        let rgb_l_v_c = Rc::clone(&rgb_l_v);
        rgb_l_v.view.connect_button_press_event(move |_, event| {
            if event.get_event_type() == gdk::EventType::ButtonPress {
                if event.get_button() == 3 {
                    rgb_l_v_c.set_selected_id(event.get_position());
                    rgb_l_v_c.popup_menu.popup_at_event(event);
                    return gtk::Inhibit(true);
                }
            };
            gtk::Inhibit(false)
        });

        rgb_l_v
    }

    fn set_selected_id(&self, posn: (f64, f64)) {
        if let Some(location) = self.view.get_path_at_pos(posn.0 as i32, posn.1 as i32) {
            if let Some(path) = location.0 {
                if let Some(list_store) = self.view.get_model() {
                    if let Some(iter) = list_store.get_iter(&path) {
                        let value = list_store.get_value(&iter, 0);
                        if let Some(string) = value.get() {
                            *self.selected_id.borrow_mut() = Some(string);
                            self.popup_menu.update_hover_condns(true);
                            return;
                        }
                    }
                }
            }
        };
        *self.selected_id.borrow_mut() = None;
        self.popup_menu.update_hover_condns(false);
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.popup_menu.update_condns(changed_condns)
    }

    pub fn connect_popup_menu_item<F: Fn(&str) + 'static>(&self, name: &str, callback: F) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    fn menu_item_selected(&self, name: &str) {
        if let Some(ref id) = *self.selected_id.borrow() {
            for callback in self
                .callbacks
                .borrow()
                .get(name)
                .expect("invalid name")
                .iter()
            {
                callback(&id)
            }
        }
    }

    pub fn add_row(&self, row: &[gtk::Value]) {
        self.list_store.append_row(&row.to_vec());
    }

    pub fn remove_row(&self, id: &str) {
        if let Some((_, iter)) = self
            .list_store
            .find_row_where(|list_store, iter| list_store.get_value(iter, 0).get() == Some(id))
        {
            self.list_store.remove(&iter);
        } else {
            panic!("{}: id not found", id);
        }
    }

    pub fn remove_all(&self) {
        self.list_store.clear();
    }
}

pub struct PaintListHelper {
    pub attributes: Vec<ScalarAttribute>,
    pub characteristics: Vec<CharacteristicType>,
}

impl PaintListHelper {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Self {
        Self {
            attributes: attributes.to_vec(),
            characteristics: characteristics.to_vec(),
        }
    }

    pub fn column_types(&self) -> Vec<gtk::Type> {
        let mut column_types = vec![
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            f64::static_type(),
        ];
        for _ in 0..self.attributes.len() * 3 + self.characteristics.len() {
            column_types.push(gtk::Type::String);
        }
        column_types
    }

    pub fn columns(&self) -> Vec<gtk::TreeViewColumn> {
        let mut cols = vec![];

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

        let mut index = 7;
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

pub trait PaintListRow: BasicPaintIfce<f64> {
    fn row(&self, helper: &PaintListHelper) -> Vec<gtk::Value> {
        let ha = if let Some(angle) = self.hue_angle() {
            angle.degrees()
        } else {
            -181.0 + self.value()
        };
        let mut row: Vec<gtk::Value> = vec![
            self.id().to_value(),
            self.rgb().pango_string().to_value(),
            self.best_foreground_rgb().pango_string().to_value(),
            self.name().or(Some("")).unwrap().to_value(),
            self.notes().or(Some("")).unwrap().to_value(),
            self.max_chroma_rgb().pango_string().to_value(),
            ha.to_value(),
        ];
        for attr in helper.attributes.iter() {
            let string = format!("{:5.4}", self.scalar_attribute(*attr));
            let attr_rgb = self.scalar_attribute_rgb(*attr);
            row.push(string.to_value());
            row.push(attr_rgb.pango_string().to_value());
            row.push(attr_rgb.best_foreground_rgb().pango_string().to_value());
        }
        for characteristic in helper.characteristics.iter() {
            let string = self.characteristic_abbrev(*characteristic);
            row.push(string.to_value());
        }
        row
    }
}

impl PaintListRow for SeriesPaint<f64> {}

impl PaintListRow for BasicPaintSpec<f64> {}
