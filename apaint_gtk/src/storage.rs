// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk::prelude::*;

use pw_gix::{
    gtkx::coloured::Colourable,
    recollections::{recall, remember},
    sav_state::{ConditionalWidgetGroups, WidgetStatesControlled, SAV_NEXT_CONDN},
    wrapper::*,
};

use crate::{colour::RGB, icon_image};
use pw_gix::sav_state::MaskedCondns;

const SAV_HAS_CURRENT_FILE: u64 = SAV_NEXT_CONDN << 0;
const SAV_TOOL_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 1;
const SAV_SESSION_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 2;
const SAV_SESSION_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 3;

const BTN_IMAGE_SIZE: i32 = 24;

#[derive(PWO, Wrapper)]
pub struct StorageManager {
    hbox: gtk::Box,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    file_name_label: gtk::Label,
    current_file_path: RefCell<Option<PathBuf>>,
    current_file_digest: RefCell<Vec<u8>>,
    load_callback: RefCell<Box<dyn Fn(&Path) -> Result<Vec<u8>, apaint::Error>>>,
    save_callback: RefCell<Box<dyn Fn(&Path) -> Result<Vec<u8>, apaint::Error>>>,
    reset_callback: RefCell<Box<dyn Fn() -> Result<Vec<u8>, apaint::Error>>>,
    last_file_key: String,
}

impl StorageManager {
    pub fn connect_load<F: Fn(&Path) -> Result<Vec<u8>, apaint::Error> + 'static>(
        &self,
        callback: F,
    ) {
        *self.load_callback.borrow_mut() = Box::new(callback);
    }

    pub fn connect_save<F: Fn(&Path) -> Result<Vec<u8>, apaint::Error> + 'static>(
        &self,
        callback: F,
    ) {
        *self.save_callback.borrow_mut() = Box::new(callback);
    }

    pub fn connect_reset<F: Fn() -> Result<Vec<u8>, apaint::Error> + 'static>(&self, callback: F) {
        *self.reset_callback.borrow_mut() = Box::new(callback);
    }

    pub fn update_session_needs_saving(&self, digest: &[u8]) {
        let condns: u64 = if digest != &self.current_file_digest.borrow()[..] {
            SAV_SESSION_NEEDS_SAVING
        } else {
            0
        };
        let mask = SAV_SESSION_NEEDS_SAVING;
        self.buttons.update_condns(MaskedCondns { condns, mask });
        self.update_file_status_button();
    }

    pub fn update_session_is_saveable(&self, is_saveable: bool) {
        let condns: u64 = if is_saveable {
            SAV_SESSION_IS_SAVEABLE
        } else {
            0
        };
        let mask = SAV_SESSION_IS_SAVEABLE;
        self.buttons.update_condns(MaskedCondns { condns, mask });
        self.update_file_status_button();
    }

    pub fn update_tool_needs_saving(&self, needs_saving: bool) {
        let condns: u64 = if needs_saving {
            SAV_TOOL_NEEDS_SAVING
        } else {
            0
        };
        let mask = SAV_TOOL_NEEDS_SAVING;
        self.buttons.update_condns(MaskedCondns { condns, mask });
        self.update_file_status_button();
    }

    pub fn needs_saving(&self) -> bool {
        let status = self.buttons.current_condns();
        status & (SAV_SESSION_NEEDS_SAVING + SAV_TOOL_NEEDS_SAVING) != 0
    }

    fn update_file_status_button(&self) {
        let current_condns = self.buttons.current_condns();
        let file_status_btn = self.buttons.get_widget("status").expect("should work");
        if current_condns & SAV_SESSION_NEEDS_SAVING != 0 {
            if current_condns & SAV_SESSION_IS_SAVEABLE != 0 {
                file_status_btn.set_image(Some(&icon_image::needs_save_ready_image(24)));
                file_status_btn.set_tooltip_text(Some(
                    "File Status: Needs Save (Ready)\nClick to save data to file",
                ));
            } else {
                file_status_btn.set_image(Some(&icon_image::needs_save_not_ready_image(24)));
                file_status_btn.set_tooltip_text(Some("File Status: Needs Save (NOT Ready)"));
            }
        } else {
            file_status_btn.set_image(Some(&icon_image::up_to_date_image(24)));
            file_status_btn.set_tooltip_text(Some("File Status: Up To Date"));
        }
    }

    fn ok_to_reset(&self) -> bool {
        let status = self.buttons.current_condns();
        if status & (SAV_SESSION_NEEDS_SAVING + SAV_TOOL_NEEDS_SAVING) != 0 {
            if status & SAV_SESSION_IS_SAVEABLE + SAV_TOOL_NEEDS_SAVING == SAV_SESSION_IS_SAVEABLE {
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
            match (self.reset_callback.borrow().as_ref())() {
                Ok(digest) => {
                    self.file_name_label.set_label("");
                    *self.current_file_path.borrow_mut() = None;
                    *self.current_file_digest.borrow_mut() = digest;
                    self.buttons.update_condns(MaskedCondns {
                        condns: 0,
                        mask: SAV_HAS_CURRENT_FILE + SAV_SESSION_NEEDS_SAVING,
                    });
                }
                Err(err) => self.report_error("Reset Error:", &err),
            };
            self.update_file_status_button();
        }
    }

    fn load(&self) {
        if self.ok_to_reset() {
            let last_file = recall(&self.last_file_key);
            let last_file = if let Some(ref text) = last_file {
                Some(text.as_str())
            } else {
                None
            };
            if let Some(path) = self.ask_file_path(Some("Load from: "), last_file, false) {
                match (self.load_callback.borrow().as_ref())(&path) {
                    Ok(digest) => {
                        remember(&self.last_file_key, &path.to_string_lossy());
                        self.file_name_label.set_label(&path.to_string_lossy());
                        *self.current_file_path.borrow_mut() = Some(path);
                        *self.current_file_digest.borrow_mut() = digest;
                        self.buttons.update_condns(MaskedCondns {
                            condns: SAV_HAS_CURRENT_FILE,
                            mask: SAV_HAS_CURRENT_FILE + SAV_SESSION_NEEDS_SAVING,
                        });
                    }
                    Err(err) => self.report_error("Load Error:", &err),
                };
                self.update_file_status_button();
            };
        }
    }

    fn save(&self) {
        if self.buttons.current_condns() & SAV_HAS_CURRENT_FILE != 0 {
            let temp = self.current_file_path.borrow();
            let path = temp.as_ref().expect("guarder");
            match (self.save_callback.borrow().as_ref())(&path) {
                Ok(digest) => {
                    *self.current_file_digest.borrow_mut() = digest;
                    self.buttons.update_condns(MaskedCondns {
                        condns: 0,
                        mask: SAV_SESSION_NEEDS_SAVING,
                    })
                }
                Err(err) => self.report_error("Save Error:", &err),
            };
            self.update_file_status_button();
        } else {
            self.save_as();
        }
    }

    fn save_as(&self) {
        let last_file = recall(&self.last_file_key);
        let last_file = if let Some(ref text) = last_file {
            Some(text.as_str())
        } else {
            None
        };
        if let Some(path) = self.ask_file_path(Some("Save as: "), last_file, false) {
            match (self.save_callback.borrow().as_ref())(&path) {
                Ok(digest) => {
                    remember(&self.last_file_key, &path.to_string_lossy());
                    self.file_name_label.set_label(&path.to_string_lossy());
                    *self.current_file_path.borrow_mut() = Some(path);
                    *self.current_file_digest.borrow_mut() = digest;
                    self.buttons.update_condns(MaskedCondns {
                        condns: SAV_HAS_CURRENT_FILE,
                        mask: SAV_HAS_CURRENT_FILE + SAV_SESSION_NEEDS_SAVING,
                    });
                }
                Err(err) => self.report_error("Save Error:", &err),
            };
            self.update_file_status_button();
        };
    }
}

pub struct StorageManagerBuilder {
    reset_tooltip_text: String,
    load_tooltip_text: String,
    save_tooltip_text: String,
    save_as_tooltip_text: String,
    last_file_key: String,
}

impl StorageManagerBuilder {
    pub fn new() -> Self {
        Self {
            reset_tooltip_text: "Reset in preparation for a new session".to_string(),
            load_tooltip_text: "Load data from a nominated file to start a new session".to_string(),
            save_tooltip_text: "Save the current session".to_string(),
            save_as_tooltip_text: "Save the current session to a new (nominated) file".to_string(),
            last_file_key: "generic".to_string(),
        }
    }

    pub fn tooltip_text(mut self, name: &str, text: &str) -> Self {
        match name {
            "reset" => self.reset_tooltip_text = text.to_string(),
            "save" => self.save_tooltip_text = text.to_string(),
            "save as" => self.save_as_tooltip_text = text.to_string(),
            "load" => self.load_tooltip_text = text.to_string(),
            _ => panic!("{}: unknown button name", name),
        };
        self
    }

    pub fn last_file_key(mut self, key: &str) -> Self {
        self.last_file_key = key.to_string();
        self
    }

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
            last_file_key: format!("{}::storage_manager::last_file", self.last_file_key),
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
