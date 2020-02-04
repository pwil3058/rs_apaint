// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use pw_gix::sav_state::{
    ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled, SAV_NEXT_CONDN,
};
use pw_gix::wrapper::*;

use colour_math::ScalarAttribute;

use apaint::series::BasicPaintSpec;

use crate::characteristics::{
    CharacteristicType, FinishEntry, FluorescenceEntry, MetallicnessEntry, PermanenceEntry,
    TransparencyEntry,
};
use crate::colour_edit::ColourEditor;

#[derive(PWO, Wrapper)]
pub struct BasicPaintSpecEditor {
    vbox: gtk::Box,
    id_entry: gtk::Entry,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    colour_editor: Rc<ColourEditor>,
    finish_entry: Rc<FinishEntry>,
    transparency_entry: Rc<TransparencyEntry>,
    permanence_entry: Rc<PermanenceEntry>,
    fluorescence_entry: Rc<FluorescenceEntry>,
    metallicness_entry: Rc<MetallicnessEntry>,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    current_spec: RefCell<Option<BasicPaintSpec<f64>>>,
    add_callbacks: RefCell<Vec<Box<dyn Fn(&BasicPaintSpec<f64>)>>>,
    accept_callbacks: RefCell<Vec<Box<dyn Fn(&str, &BasicPaintSpec<f64>)>>>,
    change_callbacks: RefCell<Vec<Box<dyn Fn(u64)>>>,
}

impl BasicPaintSpecEditor {
    pub const SAV_EDITING: u64 = SAV_NEXT_CONDN;
    pub const SAV_NOT_EDITING: u64 = SAV_NEXT_CONDN << 1;
    pub const SAV_ID_READY: u64 = SAV_NEXT_CONDN << 2;
    pub const SAV_NAME_READY: u64 = SAV_NEXT_CONDN << 3;
    pub const SAV_NOTES_READY: u64 = SAV_NEXT_CONDN << 4;
    pub const SAV_HAS_CHANGES: u64 = SAV_NEXT_CONDN << 5;
    pub const SAV_ID_CHANGED: u64 = SAV_NEXT_CONDN << 6;
    pub const SAV_NAME_CHANGED: u64 = SAV_NEXT_CONDN << 7;
    pub const SAV_NOTES_CHANGED: u64 = SAV_NEXT_CONDN << 8;
    pub const SAV_RGB_CHANGED: u64 = SAV_NEXT_CONDN << 9;

    pub const SAV_FINISH_CHANGED: u64 = SAV_NEXT_CONDN << 10;
    pub const SAV_PERMANENCE_CHANGED: u64 = SAV_NEXT_CONDN << 11;
    pub const SAV_TRANSPARENCY_CHANGED: u64 = SAV_NEXT_CONDN << 12;
    pub const SAV_FLUORESCENCE_CHANGED: u64 = SAV_NEXT_CONDN << 13;
    pub const SAV_METALLICNESS_CHANGED: u64 = SAV_NEXT_CONDN << 14;

    pub const CHANGED_MASK: u64 = Self::SAV_ID_CHANGED
        + Self::SAV_NAME_CHANGED
        + Self::SAV_NOTES_CHANGED
        + Self::SAV_RGB_CHANGED
        + Self::SAV_FINISH_CHANGED
        + Self::SAV_PERMANENCE_CHANGED
        + Self::SAV_TRANSPARENCY_CHANGED
        + Self::SAV_FLUORESCENCE_CHANGED
        + Self::SAV_METALLICNESS_CHANGED;

    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
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

        let finish_entry = FinishEntry::new();
        let transparency_entry = TransparencyEntry::new();
        let permanence_entry = PermanenceEntry::new();
        let fluorescence_entry = FluorescenceEntry::new();
        let metallicness_entry = MetallicnessEntry::new();

        let mut row: i32 = 3;
        for characteristic in characteristics.iter() {
            match *characteristic {
                CharacteristicType::Finish => {
                    grid.attach(&finish_entry.prompt(gtk::Align::End), 0, row, 1, 1);
                    grid.attach(&finish_entry.pwo(), 1, row, 1, 1);
                }
                CharacteristicType::Transparency => {
                    grid.attach(&transparency_entry.prompt(gtk::Align::End), 0, row, 1, 1);
                    grid.attach(&transparency_entry.pwo(), 1, row, 1, 1);
                }
                CharacteristicType::Permanence => {
                    grid.attach(&permanence_entry.prompt(gtk::Align::End), 0, row, 1, 1);
                    grid.attach(&permanence_entry.pwo(), 1, row, 1, 1);
                }
                CharacteristicType::Fluorescence => {
                    grid.attach(&fluorescence_entry.prompt(gtk::Align::End), 0, row, 1, 1);
                    grid.attach(&fluorescence_entry.pwo(), 1, row, 1, 1);
                }
                CharacteristicType::Metallicness => {
                    grid.attach(&metallicness_entry.prompt(gtk::Align::End), 0, row, 1, 1);
                    grid.attach(&metallicness_entry.pwo(), 1, row, 1, 1);
                }
            };
            row += 1;
        }

        let add_btn = gtk::ButtonBuilder::new().label("Add").build();
        let accept_btn = gtk::ButtonBuilder::new().label("Accept").build();
        let reset_btn = gtk::ButtonBuilder::new().label("Reset").build();
        let colour_editor = ColourEditor::new(
            attributes,
            &[add_btn.clone(), accept_btn.clone(), reset_btn.clone()],
        );
        vbox.pack_start(&colour_editor.pwo(), true, true, 0);
        let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
            WidgetStatesControlled::Sensitivity,
            None,
            None,
        );
        buttons.add_widget("add", &add_btn, Self::SAV_ID_READY + Self::SAV_NOT_EDITING);
        buttons.add_widget(
            "accept",
            &accept_btn,
            Self::SAV_ID_READY + Self::SAV_HAS_CHANGES + Self::SAV_EDITING,
        );
        buttons.add_widget("reset", &reset_btn, 0);
        let bpe = Rc::new(Self {
            vbox,
            id_entry,
            name_entry,
            notes_entry,
            colour_editor,
            finish_entry,
            transparency_entry,
            permanence_entry,
            fluorescence_entry,
            metallicness_entry,
            buttons,
            current_spec: RefCell::new(None),
            add_callbacks: RefCell::new(Vec::new()),
            accept_callbacks: RefCell::new(Vec::new()),
            change_callbacks: RefCell::new(Vec::new()),
        });

        let bpe_c = Rc::clone(&bpe);
        add_btn.connect_clicked(move |_| bpe_c.process_add_action());

        let bpe_c = Rc::clone(&bpe);
        accept_btn.connect_clicked(move |_| bpe_c.process_accept_action());

        let bpe_c = Rc::clone(&bpe);
        reset_btn.connect_clicked(move |_| bpe_c.process_reset_action());

        let bpe_c = Rc::clone(&bpe);
        bpe.id_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_ID_READY + Self::SAV_ID_CHANGED,
            };
            if entry.get_text_length() > 0 {
                masked_condns.condns += Self::SAV_ID_READY;
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.id != entry.get_text().unwrap() {
                    masked_condns.condns += Self::SAV_ID_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.name_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_NAME_READY + Self::SAV_NAME_CHANGED,
            };
            if entry.get_text_length() > 0 {
                masked_condns.condns += Self::SAV_NAME_READY;
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.name != entry.get_text().unwrap() {
                    masked_condns.condns += Self::SAV_NAME_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.notes_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_NOTES_READY + Self::SAV_NOTES_CHANGED,
            };
            if entry.get_text_length() > 0 {
                masked_condns.condns += Self::SAV_NOTES_READY;
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.notes != entry.get_text().unwrap() {
                    masked_condns.condns += Self::SAV_NOTES_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.colour_editor.connect_changed(move |rgb| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_RGB_CHANGED,
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.rgb != rgb {
                    masked_condns.condns += Self::SAV_RGB_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.finish_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_FINISH_CHANGED,
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.finish != entry.value() {
                    masked_condns.condns += Self::SAV_FINISH_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.permanence_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_PERMANENCE_CHANGED,
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.permanence != entry.value() {
                    masked_condns.condns += Self::SAV_PERMANENCE_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.transparency_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_TRANSPARENCY_CHANGED,
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.transparency != entry.value() {
                    masked_condns.condns += Self::SAV_TRANSPARENCY_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.fluorescence_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_FLUORESCENCE_CHANGED,
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.fluorescence != entry.value() {
                    masked_condns.condns += Self::SAV_FLUORESCENCE_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.metallicness_entry.connect_changed(move |entry| {
            let mut masked_condns = MaskedCondns {
                condns: 0,
                mask: Self::SAV_METALLICNESS_CHANGED,
            };
            if let Some(spec) = bpe_c.current_spec.borrow().as_ref() {
                if spec.metallicness != entry.value() {
                    masked_condns.condns += Self::SAV_METALLICNESS_CHANGED;
                }
            }
            bpe_c.buttons.update_condns(masked_condns);
            bpe_c.update_has_changes();
            bpe_c.inform_changed();
        });

        // NB: needed to correctly set the current state
        bpe.set_current_spec(None);
        bpe.update_has_changes();

        bpe
    }

    fn update_has_changes(&self) {
        let mut masked_condns = MaskedCondns {
            condns: 0,
            mask: Self::SAV_HAS_CHANGES,
        };
        if self.current_spec.borrow().is_some() {
            if self.buttons.current_condns() & Self::CHANGED_MASK != 0 {
                masked_condns.condns = Self::SAV_HAS_CHANGES;
            }
        } else if self.buttons.current_condns() & Self::SAV_ID_READY != 0 {
            masked_condns.condns = Self::SAV_HAS_CHANGES;
        }
        self.buttons.update_condns(masked_condns);
    }

    fn spec_from_entries(&self) -> BasicPaintSpec<f64> {
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
        paint_spec.finish = self.finish_entry.value();
        paint_spec.permanence = self.permanence_entry.value();
        paint_spec.transparency = self.transparency_entry.value();
        paint_spec.fluorescence = self.fluorescence_entry.value();
        paint_spec.metallicness = self.metallicness_entry.value();
        paint_spec
    }

    fn process_add_action(&self) {
        let paint_spec = self.spec_from_entries();
        self.set_current_spec(Some(&paint_spec));
        self.update_has_changes();
        for callback in self.add_callbacks.borrow().iter() {
            callback(&paint_spec);
        }
    }

    fn process_accept_action(&self) {
        let edited_spec = self
            .current_spec
            .borrow()
            .clone()
            .expect("programming error");
        let paint_spec = self.spec_from_entries();
        self.set_current_spec(Some(&paint_spec));
        self.update_has_changes();
        for callback in self.accept_callbacks.borrow().iter() {
            callback(&edited_spec.id, &paint_spec);
        }
    }

    fn process_reset_action(&self) {
        if self.buttons.current_condns() & Self::SAV_HAS_CHANGES != 0 {
            if self.buttons.current_condns() & Self::SAV_ID_READY != 0 {
                let buttons = &[
                    ("Cancel", gtk::ResponseType::Other(0)),
                    ("Save and Continue", gtk::ResponseType::Other(1)),
                    ("Continue Discarding Changes", gtk::ResponseType::Other(2)),
                ];
                match self.ask_question("There are unsaved changes!", None, buttons) {
                    gtk::ResponseType::Other(0) => return,
                    gtk::ResponseType::Other(1) => {
                        if self.buttons.current_condns() & Self::SAV_EDITING != 0 {
                            self.process_accept_action()
                        } else {
                            self.process_add_action()
                        }
                    }
                    _ => (),
                }
            } else {
                let buttons = &[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Continue Discarding Changes", gtk::ResponseType::Accept),
                ];
                if self.ask_question("There are unsaved changes!", None, buttons)
                    == gtk::ResponseType::Cancel
                {
                    return;
                }
            }
        }
        self.set_current_spec(None);
        self.id_entry.set_text("");
        self.name_entry.set_text("");
        self.notes_entry.set_text("");
        // NB: do not reset characteristics
        self.colour_editor.reset();
        self.update_has_changes();
    }

    fn set_current_spec(&self, spec: Option<&BasicPaintSpec<f64>>) {
        let mut masked_condns = MaskedCondns {
            condns: 0,
            mask: Self::SAV_EDITING + Self::SAV_NOT_EDITING + Self::CHANGED_MASK,
        };
        if let Some(spec) = spec {
            *self.current_spec.borrow_mut() = Some(spec.clone());
            masked_condns.condns = Self::SAV_EDITING;
        } else {
            *self.current_spec.borrow_mut() = None;
            masked_condns.condns = Self::SAV_NOT_EDITING;
        };
        self.buttons.update_condns(masked_condns);
    }

    pub fn edit(&self, spec: &BasicPaintSpec<f64>) {
        self.set_current_spec(Some(spec));
        self.id_entry.set_text(&spec.id);
        self.name_entry.set_text(&spec.name);
        self.notes_entry.set_text(&spec.notes);
        self.colour_editor.set_rgb(spec.rgb);
        self.finish_entry.set_value(Some(spec.finish));
        self.permanence_entry.set_value(Some(spec.permanence));
        self.transparency_entry.set_value(Some(spec.transparency));
        self.fluorescence_entry.set_value(Some(spec.fluorescence));
        self.metallicness_entry.set_value(Some(spec.metallicness));
        self.update_has_changes();
    }

    pub fn un_edit(&self, id: &str) {
        let is_being_edited = if let Some(spec) = self.current_spec.borrow().as_ref() {
            id == spec.id
        } else {
            false
        };
        if is_being_edited {
            self.set_current_spec(None);
            self.update_has_changes();
        }
    }

    pub fn connect_add_action<F: Fn(&BasicPaintSpec<f64>) + 'static>(&self, callback: F) {
        self.add_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn connect_accept_action<F: Fn(&str, &BasicPaintSpec<f64>) + 'static>(&self, callback: F) {
        self.accept_callbacks.borrow_mut().push(Box::new(callback))
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
        self.set_current_spec(None);
        self.id_entry.set_text("");
        self.name_entry.set_text("");
        self.notes_entry.set_text("");
        self.finish_entry.set_value(None);
        self.permanence_entry.set_value(None);
        self.transparency_entry.set_value(None);
        self.fluorescence_entry.set_value(None);
        self.metallicness_entry.set_value(None);
        self.colour_editor.reset();
        self.update_has_changes();
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.buttons.current_condns() & Self::SAV_HAS_CHANGES != 0
    }
}
