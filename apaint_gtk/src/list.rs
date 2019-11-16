// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;

use apaint_gtk_boilerplate::PWO;
use pw_gix::{gtkx::list_store::ListRowOps, wrapper::PackableWidgetObject};

use crate::colour::{ColourInterface, RGB};

#[derive(PWO)]
pub struct RGBListView {
    view: gtk::TreeView,
    selected_id: RefCell<Option<String>>,
}

impl RGBListView {
    pub fn new() -> Rc<Self> {
        let list_store =
            gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String, gtk::Type::String]);
        for rgb in RGB::PRIMARIES
            .iter()
            .chain(RGB::SECONDARIES.iter())
            .chain(RGB::GREYS.iter())
        {
            let row: Vec<gtk::Value> = vec![
                rgb.pango_string().to_value(),
                rgb.pango_string().to_value(),
                rgb.best_foreground_rgb().pango_string().to_value(),
            ];
            list_store.append_row(&row);
        }

        let view = gtk::TreeViewBuilder::new().headers_visible(true).build();
        view.set_model(Some(&list_store));
        view.get_selection().set_mode(gtk::SelectionMode::None);

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
        view.append_column(&col);

        let rgb_l_v = Rc::new(Self {
            view,
            selected_id: RefCell::new(None),
        });

        let rgb_l_v_c = Rc::clone(&rgb_l_v);
        rgb_l_v.view.connect_button_press_event(move |_, event| {
            if event.get_event_type() == gdk::EventType::ButtonPress {
                if event.get_button() == 3 {
                    rgb_l_v_c.set_selected_id(event.get_position());
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
                            return;
                        }
                    }
                }
            }
        };
        *self.selected_id.borrow_mut() = None;
    }
}
