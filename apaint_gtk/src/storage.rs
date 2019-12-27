// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk::prelude::*;

use pw_gix::{
    gtkx::coloured::Colourable,
    sav_state::{ConditionalWidgetGroups, WidgetStatesControlled, SAV_NEXT_CONDN},
    wrapper::*,
};

use apaint_gtk_boilerplate::{Wrapper, PWO};

use crate::{colour::RGB, icon_image};

pub const SAV_HAS_CURRENT_FILE: u64 = SAV_NEXT_CONDN << 0;
pub const SAV_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 1;
pub const SAV_TOOL_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 2;
pub const SAV_SESSION_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 3;
pub const SAV_SESSION_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 4;

const BTN_IMAGE_SIZE: i32 = 24;

#[derive(PWO, Wrapper)]
pub struct StorageManager {
    hbox: gtk::Box,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    file_name_label: gtk::Label,
    current_file_path: RefCell<Option<PathBuf>>,
    current_file_digest: RefCell<Vec<u8>>,
    load_callback: RefCell<Box<dyn Fn(&Path) -> Result<(), apaint::Error>>>,
    save_callback: RefCell<Box<dyn Fn(&Path) -> Result<(), apaint::Error>>>,
    reset_callback: RefCell<Box<dyn Fn() -> Result<(), apaint::Error>>>,
}

impl StorageManager {
    fn ok_to_reset(&self) -> bool {
        let status = self.buttons.current_condns();
        if status & (SAV_SESSION_NEEDS_SAVING + SAV_TOOL_NEEDS_SAVING) != 0 {
            if status & SAV_IS_SAVEABLE != 0 {
                let buttons = [
                    ("Cancel", gtk::ResponseType::Other(0)),
                    ("Save and Continue", gtk::ResponseType::Other(1)),
                    ("Continue Discarding Changes", gtk::ResponseType::Other(2)),
                ];
                match self.ask_question("There are unsaved changes!", None, &buttons) {
                    gtk::ResponseType::Other(0) => return false,
                    gtk::ResponseType::Other(1) => {
                        let o_path = self.current_file_path.borrow().clone();
                        if let Some(path) = o_path {
                            if let Err(err) = (self.save_callback.borrow().as_ref())(&path) {
                                self.report_error("Failed to save file", &err);
                                return false;
                            }
                        } else if let Some(path) =
                            self.ask_file_path(Some("Save as: "), None, false)
                        {
                            if let Err(err) = (self.save_callback.borrow().as_ref())(&path) {
                                self.report_error("Failed to save file", &err);
                                return false;
                            }
                        } else {
                            return false;
                        };
                        return true;
                    }
                    _ => return true,
                }
            } else {
                let buttons = &[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Continue Discarding Changes", gtk::ResponseType::Accept),
                ];
                return self.ask_question("There are unsaved changes!", None, buttons)
                    == gtk::ResponseType::Accept;
            }
        };
        true
    }

    fn reset(&self) {
        if self.ok_to_reset() {
            if let Err(err) = (self.reset_callback.borrow().as_ref())() {
                self.report_error("Reset Error:", &err);
            };
        }
    }

    fn load(&self) {
        if self.ok_to_reset() {
            // TODO: use last dir data option
            if let Some(path) = self.ask_file_path(Some("Load from: "), None, false) {
                if let Err(err) = (self.load_callback.borrow().as_ref())(&path) {
                    self.report_error("Load Error:", &err);
                };
            };
        }
    }

    fn save(&self) {
        let temp = self.current_file_path.borrow();
        if let Some(path) = temp.as_ref() {
            if let Err(err) = (self.save_callback.borrow().as_ref())(&path) {
                self.report_error("Save Error:", &err);
            };
        } else {
            self.save_as();
        }
    }

    fn save_as(&self) {
        // TODO: use last dir data option
        if let Some(path) = self.ask_file_path(Some("Save as: "), None, false) {
            if let Err(err) = (self.save_callback.borrow().as_ref())(&path) {
                self.report_error("Save Error:", &err);
            };
        };
    }
}

pub struct StorageManagerBuilder {
    reset_tooltip_text: String,
    load_tooltip_text: String,
    save_tooltip_text: String,
    save_as_tooltip_text: String,
}

impl StorageManagerBuilder {
    pub fn build(self) -> Rc<StorageManager> {
        let storage_manager = Rc::new(StorageManager {
            hbox: gtk::Box::new(gtk::Orientation::Horizontal, 0),
            buttons: ConditionalWidgetGroups::<gtk::Button>::new(
                WidgetStatesControlled::Sensitivity,
                None,
                None,
            ),
            file_name_label: gtk::LabelBuilder::new()
                .justify(gtk::Justification::Left)
                .xalign(0.01)
                .build(),
            current_file_path: RefCell::new(None),
            current_file_digest: RefCell::new(vec![]),
            reset_callback: RefCell::new(Box::new(|| Err(apaint::Error::NotImplemented))),
            save_callback: RefCell::new(Box::new(|_| Err(apaint::Error::NotImplemented))),
            load_callback: RefCell::new(Box::new(|_| Err(apaint::Error::NotImplemented))),
        });

        // Reset
        let button = gtk::ButtonBuilder::new()
            .tooltip_text(&self.reset_tooltip_text)
            .image(&icon_image::colln_new_image(BTN_IMAGE_SIZE).upcast::<gtk::Widget>())
            .build();
        storage_manager.buttons.add_widget("reset", &button, 0);
        storage_manager.hbox.pack_start(&button, false, false, 0);
        let sm_c = Rc::clone(&storage_manager);
        button.connect_clicked(move |_| sm_c.reset());

        // Load
        let button = gtk::ButtonBuilder::new()
            .tooltip_text(&self.load_tooltip_text)
            .image(&icon_image::colln_load_image(BTN_IMAGE_SIZE).upcast::<gtk::Widget>())
            .build();
        storage_manager.buttons.add_widget("load", &button, 0);
        storage_manager.hbox.pack_start(&button, false, false, 0);
        let sm_c = Rc::clone(&storage_manager);
        button.connect_clicked(move |_| sm_c.load());

        // Save
        let button = gtk::ButtonBuilder::new()
            .tooltip_text(&self.save_tooltip_text)
            .image(&icon_image::colln_save_image(BTN_IMAGE_SIZE).upcast::<gtk::Widget>())
            .build();
        storage_manager
            .buttons
            .add_widget("save", &button, SAV_SESSION_IS_SAVEABLE);
        storage_manager.hbox.pack_start(&button, false, false, 0);
        let sm_c = Rc::clone(&storage_manager);
        button.connect_clicked(move |_| sm_c.save());

        // Save As
        let button = gtk::ButtonBuilder::new()
            .tooltip_text(&self.save_as_tooltip_text)
            .image(&icon_image::colln_save_as_image(BTN_IMAGE_SIZE).upcast::<gtk::Widget>())
            .build();
        storage_manager.buttons.add_widget(
            "save as",
            &button,
            SAV_SESSION_IS_SAVEABLE + SAV_HAS_CURRENT_FILE,
        );
        storage_manager.hbox.pack_start(&button, false, false, 0);
        let sm_c = Rc::clone(&storage_manager);
        button.connect_clicked(move |_| sm_c.save_as());

        storage_manager
            .hbox
            .pack_start(&gtk::Label::new(Some("Current File:")), false, false, 1);
        storage_manager
            .file_name_label
            .set_widget_colour_rgb(RGB::WHITE);
        storage_manager
            .hbox
            .pack_start(&storage_manager.file_name_label, true, true, 1);

        let button = gtk::ButtonBuilder::new().sensitive(false).build();
        button.set_image(Some(&icon_image::up_to_date_image(BTN_IMAGE_SIZE)));
        storage_manager.hbox.pack_start(&button, false, false, 1);
        storage_manager.buttons.add_widget(
            "status",
            &button,
            SAV_SESSION_IS_SAVEABLE + SAV_SESSION_NEEDS_SAVING,
        );
        let sm_c = Rc::clone(&storage_manager);
        button.connect_clicked(move |_| sm_c.save());

        storage_manager
    }
}
