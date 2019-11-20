// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gtk::prelude::*;

use apaint_gtk_boilerplate::PWO;
use pw_gix::{
    gtkx::{list_store::ListRowOps, menu::ManagedMenu, tree_model::TreeModelRowOps},
    sav_state::MaskedCondns,
    sav_state::WidgetStatesControlled,
    wrapper::PackableWidgetObject,
};

use crate::colour::{ColourInterface, ScalarAttribute, RGB};
use apaint::characteristics::CharacteristicType;
use apaint::BasicPaintIfce;

#[derive(PWO)]
pub struct ColouredItemListView {
    view: gtk::TreeView,
    selected_id: RefCell<Option<String>>,
    popup_menu: ManagedMenu,
    callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(&str)>>>>,
}

impl ColouredItemListView {
    pub const SAV_SELECTION_MADE: u64 = 1;

    pub fn new(
        list_store: &gtk::ListStore,
        columns: &[gtk::TreeViewColumn],
        menu_items: &'static [(&str, &str, Option<&gtk::Image>, &str, u64)],
    ) -> Rc<Self> {
        let view = gtk::TreeViewBuilder::new().headers_visible(true).build();
        view.set_model(Some(list_store));
        view.get_selection().set_mode(gtk::SelectionMode::None);

        for col in columns.iter() {
            view.append_column(col);
        }

        let rgb_l_v = Rc::new(Self {
            view,
            selected_id: RefCell::new(None),
            popup_menu: ManagedMenu::new(WidgetStatesControlled::Sensitivity, None, None, &[]),
            callbacks: RefCell::new(HashMap::new()),
        });

        for &(name, label_text, image, tooltip_text, condns) in menu_items.iter() {
            let rgb_l_v_c = Rc::clone(&rgb_l_v);
            rgb_l_v
                .popup_menu
                .append_item(name, label_text, image, tooltip_text, condns)
                .connect_activate(move |_| rgb_l_v_c.menu_item_selected(name));
            rgb_l_v
                .callbacks
                .borrow_mut()
                .insert(name.to_string(), vec![]);
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
                            self.popup_menu.update_condns(MaskedCondns {
                                condns: Self::SAV_SELECTION_MADE,
                                mask: Self::SAV_SELECTION_MADE,
                            });
                            return;
                        }
                    }
                }
            }
        };
        *self.selected_id.borrow_mut() = None;
        self.popup_menu.update_condns(MaskedCondns {
            condns: 0,
            mask: Self::SAV_SELECTION_MADE,
        });
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

    pub fn set_list_store(&self, new_list_store: &gtk::ListStore) {
        self.view.set_model(Some(new_list_store));
    }
}

pub struct RGBListHelper {
    pub attributes: Vec<ScalarAttribute>,
    pub column_types: Vec<gtk::Type>,
}

impl RGBListHelper {
    pub fn new(attributes: &[ScalarAttribute]) -> Self {
        let mut column_types = vec![
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            f64::static_type(),
        ];
        for _ in 0..attributes.len() * 3 {
            column_types.push(gtk::Type::String);
        }
        Self {
            attributes: attributes.to_vec(),
            column_types,
        }
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
            .title("Hue")
            .sort_column_id(4)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "background", 3);
        cols.push(col);

        let mut index = 5;
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

        cols
    }

    pub fn row(&self, rgb: &RGB) -> Vec<gtk::Value> {
        let ha = if let Some(angle) = rgb.hue_angle() {
            angle.degrees()
        } else {
            -181.0 + rgb.value()
        };
        let mut row: Vec<gtk::Value> = vec![
            rgb.pango_string().to_value(),
            rgb.pango_string().to_value(),
            rgb.best_foreground_rgb().pango_string().to_value(),
            rgb.max_chroma_rgb().pango_string().to_value(),
            ha.to_value(),
        ];
        for attr in self.attributes.iter() {
            // TODO: add a scalar_attribute_rgb() method to colour interface
            let string = format!("{:5.4}", rgb.scalar_attribute(*attr));
            let attr_rgb = rgb.scalar_attribute_rgb(*attr);
            row.push(string.to_value());
            row.push(attr_rgb.pango_string().to_value());
            row.push(attr_rgb.best_foreground_rgb().pango_string().to_value());
        }
        row
    }

    pub fn new_list_store(&self, rgbs: &[RGB]) -> gtk::ListStore {
        let list_store = gtk::ListStore::new(&self.column_types);
        for rgb in rgbs.iter() {
            let row = self.row(rgb);
            list_store.append_row(&row);
        }
        list_store
    }
}

#[derive(PWO)]
pub struct RGBList {
    scrolled_window: gtk::ScrolledWindow,
    ci_list_view: Rc<ColouredItemListView>,
    list_store: RefCell<gtk::ListStore>,
    list_helper: RGBListHelper,
}

impl RGBList {
    pub fn new(contents: &[RGB], attributes: &[ScalarAttribute]) -> Rc<Self> {
        let list_helper = RGBListHelper::new(attributes);
        let list_store = list_helper.new_list_store(contents);

        let ci_list_view = ColouredItemListView::new(
            &list_store,
            &list_helper.columns(),
            &[(
                "info",
                "Colour Information",
                None,
                "Show detailed information for colour under the pointer",
                ColouredItemListView::SAV_SELECTION_MADE,
            )],
        );
        ci_list_view.connect_popup_menu_item("info", |id| println!("info requested for '{}'", id));
        let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
        scrolled_window.add(&ci_list_view.pwo());

        Rc::new(Self {
            scrolled_window,
            ci_list_view,
            list_store: RefCell::new(list_store),
            list_helper,
        })
    }

    pub fn add_rgb(&self, rgb: &RGB) {
        let row = self.list_helper.row(rgb);
        self.list_store.borrow().append_row(&row);
    }

    pub fn remove_rgb(&self, id: &str) {
        let list_store = self.list_store.borrow();
        if let Some((_, iter)) =
            list_store.find_row_where(|store, iter| store.get_value(iter, 0).get() == Some(id))
        {
            list_store.remove(&iter);
        }
    }

    pub fn set_contents(&self, rgbs: &[RGB]) {
        *self.list_store.borrow_mut() = self.list_helper.new_list_store(rgbs);
        self.ci_list_view.set_list_store(&self.list_store.borrow());
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
            .title("Hue")
            .sort_column_id(4)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "background", 3);
        cols.push(col);

        let mut index = 5;
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
                .title(&characteristic.to_string())
                .sort_column_id(index)
                .sort_indicator(true)
                .build();
            let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
            col.pack_start(&cell, false);
            col.add_attribute(&cell, "text", index);
            col.add_attribute(&cell, "background", 1);
            col.add_attribute(&cell, "foreground", 2);
            cols.push(col);
            index += 3;
        }

        cols
    }

    pub fn row(&self, paint: &Rc<dyn BasicPaintIfce<f64>>) -> Vec<gtk::Value> {
        let ha = if let Some(angle) = paint.hue_angle() {
            angle.degrees()
        } else {
            -181.0 + paint.value()
        };
        let mut row: Vec<gtk::Value> = vec![
            paint.rgb().pango_string().to_value(),
            paint.rgb().pango_string().to_value(),
            paint.best_foreground_rgb().pango_string().to_value(),
            paint.max_chroma_rgb().pango_string().to_value(),
            ha.to_value(),
        ];
        for attr in self.attributes.iter() {
            // TODO: add a scalar_attribute_rgb() method to colour interface
            let string = format!("{:5.4}", paint.scalar_attribute(*attr));
            let attr_rgb = paint.scalar_attribute_rgb(*attr);
            row.push(string.to_value());
            row.push(attr_rgb.pango_string().to_value());
            row.push(attr_rgb.best_foreground_rgb().pango_string().to_value());
        }
        for characteristic in self.characteristics.iter() {
            let string = paint.characteristic_abbrev(*characteristic);
            row.push(string.to_value());
        }
        row
    }

    pub fn new_list_store(&self, paints: &[Rc<dyn BasicPaintIfce<f64>>]) -> gtk::ListStore {
        let list_store = gtk::ListStore::new(&self.column_types());
        for paint in paints.iter() {
            let row = self.row(paint);
            list_store.append_row(&row);
        }
        list_store
    }
}
