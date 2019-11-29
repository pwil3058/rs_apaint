// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use pw_gix::sav_state::{ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled};
use pw_gix::wrapper::*;

use colour_math::ScalarAttribute;

use apaint::{characteristics::CharacteristicType, BasicPaintSpec};

use apaint_gtk_boilerplate::PWO;

use crate::colour_edit::ColourEditor;

#[derive(PWO)]
pub struct BasicPaintSpecEditor {
    vbox: gtk::Box,
    id_entry: gtk::Entry,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    colour_editor: Rc<ColourEditor>,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    add_callbacks: RefCell<Vec<Box<dyn Fn(&BasicPaintSpec<f64>)>>>,
    change_callbacks: RefCell<Vec<Box<dyn Fn(u64)>>>,
}

impl BasicPaintSpecEditor {
    pub const SAV_ID_READY: u64 = 1 << 0;
    pub const SAV_NAME_READY: u64 = 1 << 1;
    pub const SAV_NOTES_READY: u64 = 1 << 2;

    pub fn new(
        attributes: &[ScalarAttribute],
        _characteristics: &[CharacteristicType],
    ) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let grid = gtk::GridBuilder::new().hexpand(true).build();
        vbox.pack_start(&grid, false, false, 0);
        let label = gtk::LabelBuilder::new()
            .label("Id:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 0, 1, 1);
        let id_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&id_entry, 1, 0, 1, 1);
        let label = gtk::LabelBuilder::new()
            .label("Name:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 1, 1, 1);
        let name_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&name_entry, 1, 1, 1, 1);
        let label = gtk::LabelBuilder::new()
            .label("Notes:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 2, 1, 1);
        let notes_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&notes_entry, 1, 2, 1, 1);
        let add_btn = gtk::ButtonBuilder::new().label("Add").build();
        let colour_editor = ColourEditor::new(attributes, &[add_btn.clone()]);
        vbox.pack_start(&colour_editor.pwo(), true, true, 0);
        let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
            WidgetStatesControlled::Sensitivity,
            None,
            None,
        );
        buttons.add_widget("add", &add_btn, Self::SAV_ID_READY);
        let bpe = Rc::new(Self {
            vbox,
            id_entry,
            name_entry,
            notes_entry,
            colour_editor,
            buttons,
            add_callbacks: RefCell::new(Vec::new()),
            change_callbacks: RefCell::new(Vec::new()),
        });

        let bpe_c = Rc::clone(&bpe);
        add_btn.connect_clicked(move |_| bpe_c.process_add_action());

        let bpe_c = Rc::clone(&bpe);
        bpe.id_entry.connect_changed(move |entry| {
            if entry.get_text_length() > 0 {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: Self::SAV_ID_READY,
                    mask: Self::SAV_ID_READY,
                })
            } else {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: 0,
                    mask: Self::SAV_ID_READY,
                })
            }
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.name_entry.connect_changed(move |entry| {
            if entry.get_text_length() > 0 {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: Self::SAV_NAME_READY,
                    mask: Self::SAV_NAME_READY,
                })
            } else {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: 0,
                    mask: Self::SAV_NAME_READY,
                })
            }
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.notes_entry.connect_changed(move |entry| {
            let condns = if entry.get_text_length() > 0 {
                Self::SAV_NOTES_READY
            } else {
                0
            };
            bpe_c.buttons.update_condns(MaskedCondns {
                condns,
                mask: Self::SAV_NOTES_READY,
            });
        });

        bpe
    }

    fn process_add_action(&self) {
        let id = self
            .id_entry
            .get_text()
            .expect("shouldn't be called otherwise");
        let rgb = self.colour_editor.rgb();
        let mut paint_spec = BasicPaintSpec::new(rgb, &id);
        if let Some(name) = self.name_entry.get_text() {
            paint_spec.name = name.to_string();
        }
        if let Some(notes) = self.notes_entry.get_text() {
            paint_spec.notes = notes.to_string();
        }
        for callback in self.add_callbacks.borrow().iter() {
            callback(&paint_spec);
        }
    }

    pub fn connect_add_action<F: Fn(&BasicPaintSpec<f64>) + 'static>(&self, callback: F) {
        self.add_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_changed(&self) {
        let status = self.buttons.current_condns();
        for callback in self.change_callbacks.borrow().iter() {
            callback(status)
        }
    }

    pub fn connect_changed<F: Fn(u64) + 'static>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn hard_reset(&self) {
        self.id_entry.set_text("");
        self.name_entry.set_text("");
        self.notes_entry.set_text("");
        // TODO: reset characteristics
        self.colour_editor.reset();
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.buttons.current_condns() & Self::SAV_ID_READY != 0
    }
}
