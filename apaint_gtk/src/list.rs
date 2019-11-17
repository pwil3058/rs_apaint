// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gtk::prelude::*;

use apaint_gtk_boilerplate::PWO;
use pw_gix::{
    gtkx::{list_store::ListRowOps, menu::ManagedMenu},
    sav_state::WidgetStatesControlled,
    wrapper::PackableWidgetObject,
};

use crate::colour::{ColourInterface, RGB};
use pw_gix::sav_state::MaskedCondns;

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
}

#[derive(PWO)]
pub struct RGBList {
    scrolled_window: gtk::ScrolledWindow,
    _list_store: gtk::ListStore,
}

impl RGBList {
    pub fn new() -> Rc<Self> {
        let _list_store = gtk::ListStore::new(&[
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            gtk::Type::String,
            f64::static_type(),
        ]);
        for rgb in RGB::PRIMARIES
            .iter()
            .chain(RGB::SECONDARIES.iter())
            .chain(RGB::GREYS.iter())
        {
            let ha = if let Some(angle) = rgb.hue_angle() {
                angle.degrees()
            } else {
                -181.0 + rgb.value()
            };
            let row: Vec<gtk::Value> = vec![
                rgb.pango_string().to_value(),
                rgb.pango_string().to_value(),
                rgb.best_foreground_rgb().pango_string().to_value(),
                rgb.max_chroma_rgb().pango_string().to_value(),
                ha.to_value(),
            ];
            _list_store.append_row(&row);
        }

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

        let col2 = gtk::TreeViewColumnBuilder::new()
            .title("Hue")
            .sort_column_id(4)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col2.pack_start(&cell, false);
        col2.add_attribute(&cell, "background", 3);

        let ci_list_view = ColouredItemListView::new(
            &_list_store,
            &[col, col2],
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
            _list_store,
        })
    }
}
