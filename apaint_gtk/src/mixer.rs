// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod component;
pub mod targeted;

pub mod saved {
    use std::{
        cell::RefCell,
        path::{Path, PathBuf},
        rc::Rc,
    };

    use gtk::prelude::*;

    use pw_gix::{
        gtkx::coloured::Colourable,
        sav_state::{
            ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled, SAV_NEXT_CONDN,
        },
        wrapper::*,
    };

    use crate::{colour::RGB, icon_image};

    #[derive(PWO, Wrapper)]
    pub struct MixerFileManagerObsolete {
        hbox: gtk::Box,
        buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
        file_name_label: gtk::Label,
        file_status_btn: gtk::Button,
        current_file_path: RefCell<Option<PathBuf>>,
        current_file_digest: RefCell<Vec<u8>>,
        load_file_callback: RefCell<Option<Box<dyn Fn(&Path) -> Result<(), apaint::Error>>>>,
        write_file_callback: RefCell<Option<Box<dyn Fn(&Path) -> Result<(), apaint::Error>>>>,
        reset_callback: RefCell<Option<Box<dyn Fn() -> Result<(), apaint::Error>>>>,
    }

    impl MixerFileManagerObsolete {
        pub const SAV_HAS_CURRENT_FILE: u64 = SAV_NEXT_CONDN << 0;
        pub const SAV_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 1;
        pub const SAV_TOOL_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 2;
        pub const SAV_SESSION_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 3;
        pub const SAV_SESSION_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 4;

        const BTN_IMAGE_SIZE: i32 = 24;

        pub fn new() -> Rc<Self> {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
                WidgetStatesControlled::Sensitivity,
                None,
                None,
            );

            let reset_btn = gtk::ButtonBuilder::new()
                .tooltip_text("Clear the mixer in preparation for creating a new mixing session")
                .build();
            // TODO: change setting of image when ButtonBuilder interface is fixed.
            reset_btn.set_image(Some(&icon_image::colln_new_image(Self::BTN_IMAGE_SIZE)));
            buttons.add_widget("new_colln", &reset_btn, 0);
            hbox.pack_start(&reset_btn, false, false, 0);

            let load_colln_btn = gtk::ButtonBuilder::new()
                .tooltip_text("Load a saved mixing session.")
                .build();
            // TODO: change setting of image when ButtonBuilder interface is fixed.
            load_colln_btn.set_image(Some(&icon_image::colln_load_image(Self::BTN_IMAGE_SIZE)));
            buttons.add_widget("load_colln", &load_colln_btn, 0);
            hbox.pack_start(&load_colln_btn, false, false, 0);

            let save_colln_btn = gtk::ButtonBuilder::new()
                .tooltip_text("Save the current mixing session to the current file.")
                .build();
            // TODO: change setting of image when ButtonBuilder interface is fixed.
            save_colln_btn.set_image(Some(&icon_image::colln_save_image(Self::BTN_IMAGE_SIZE)));
            buttons.add_widget(
                "save_colln",
                &save_colln_btn,
                Self::SAV_HAS_CURRENT_FILE + Self::SAV_SESSION_IS_SAVEABLE,
            );
            hbox.pack_start(&save_colln_btn, false, false, 0);

            let save_as_colln_btn = gtk::ButtonBuilder::new()
                .tooltip_text("Save the current mixing session to a nominated file.")
                .build();
            // TODO: change setting of image when ButtonBuilder interface is fixed.
            save_as_colln_btn
                .set_image(Some(&icon_image::colln_save_as_image(Self::BTN_IMAGE_SIZE)));
            buttons.add_widget(
                "save_as_colln",
                &save_as_colln_btn,
                Self::SAV_SESSION_IS_SAVEABLE,
            );
            hbox.pack_start(&save_as_colln_btn, false, false, 0);

            hbox.pack_start(&gtk::Label::new(Some("Current File:")), false, false, 1);
            let file_name_label = gtk::LabelBuilder::new()
                .justify(gtk::Justification::Left)
                .xalign(0.01)
                .build();
            file_name_label.set_widget_colour_rgb(RGB::WHITE);
            hbox.pack_start(&file_name_label, true, true, 1);

            let file_status_btn = gtk::ButtonBuilder::new().sensitive(false).build();
            file_status_btn.set_image(Some(&icon_image::up_to_date_image(Self::BTN_IMAGE_SIZE)));
            hbox.pack_start(&file_status_btn, false, false, 1);
            buttons.add_widget(
                "file_status",
                &file_status_btn,
                Self::SAV_SESSION_IS_SAVEABLE + Self::SAV_SESSION_NEEDS_SAVING,
            );

            hbox.show_all();

            let mfm = Rc::new(Self {
                hbox,
                buttons,
                file_name_label,
                file_status_btn,
                current_file_path: RefCell::new(None),
                current_file_digest: RefCell::new(vec![]),
                load_file_callback: RefCell::new(None),
                write_file_callback: RefCell::new(None),
                reset_callback: RefCell::new(None),
            });

            let mfm_c = Rc::clone(&mfm);
            load_colln_btn.connect_clicked(move |_|
                // TODO: use last dir data option
                if mfm_c.ok_to_reset() {
                    if let Some(path) = mfm_c.ask_file_path(Some("Load from: "), None, false) {
                        if let Some(callback) = &*mfm_c.write_file_callback.borrow() {
                            if let Err(err) = callback(&path) {
                                mfm_c.report_error("Problem loading file", &err);
                            };
                        }
                    }
                }
            );

            let mfm_c = Rc::clone(&mfm);
            save_as_colln_btn.connect_clicked(move |_|
                // TODO: use last dir data option
                if let Some(path) = mfm_c.ask_file_path(Some("Save as: "), None, false) {
                    if let Some(callback) = &*mfm_c.write_file_callback.borrow() {
                        if let Err(err) = callback(&path) {
                            mfm_c.report_error("Problem saving file", &err);
                        };
                    }
                }
            );

            let mfm_c = Rc::clone(&mfm);
            save_colln_btn.connect_clicked(move |_| {
                if let Some(callback) = &*mfm_c.write_file_callback.borrow() {
                    let path = mfm_c
                        .current_file_path
                        .borrow()
                        .clone()
                        .expect("shouldn't be callable");
                    if let Err(err) = callback(&path) {
                        mfm_c.report_error("Problem saving file", &err);
                    };
                }
            });

            let mfm_c = Rc::clone(&mfm);
            reset_btn.connect_clicked(move |_| {
                if mfm_c.ok_to_reset() {
                    if let Some(callback) = &*mfm_c.reset_callback.borrow() {
                        if let Err(err) = callback() {
                            mfm_c.report_error("Problem resetting", &err);
                        };
                    }
                }
            });

            mfm
        }

        pub fn set_current_file_path<Q: AsRef<Path>>(&self, path: Option<(Q, &[u8])>) {
            let mut condns: u64 = 0;
            let mask: u64 = Self::SAV_HAS_CURRENT_FILE + Self::SAV_SESSION_NEEDS_SAVING;
            if let Some((path, digest)) = path {
                let path: PathBuf = path.as_ref().to_path_buf();
                self.file_name_label.set_label(&path.to_string_lossy());
                *self.current_file_path.borrow_mut() = Some(path);
                *self.current_file_digest.borrow_mut() = digest.to_vec();
                condns = Self::SAV_HAS_CURRENT_FILE;
            } else {
                self.file_name_label.set_label("");
                *self.current_file_path.borrow_mut() = None;
                *self.current_file_digest.borrow_mut() = vec![];
            }
            self.update_condns(MaskedCondns { condns, mask });
        }

        fn update_file_status_button(&self) {
            let current_condns = self.buttons.current_condns();
            if current_condns & Self::SAV_SESSION_NEEDS_SAVING != 0 {
                if current_condns & Self::SAV_SESSION_IS_SAVEABLE != 0 {
                    self.file_status_btn
                        .set_image(Some(&icon_image::needs_save_ready_image(24)));
                    self.file_status_btn.set_tooltip_text(Some(
                        "File Status: Needs Save (Ready)\nClick to save data to file",
                    ));
                } else {
                    self.file_status_btn
                        .set_image(Some(&icon_image::needs_save_not_ready_image(24)));
                    self.file_status_btn
                        .set_tooltip_text(Some("File Status: Needs Save (NOT Ready)"));
                }
            } else {
                self.file_status_btn
                    .set_image(Some(&icon_image::up_to_date_image(24)));
                self.file_status_btn
                    .set_tooltip_text(Some("File Status: Up To Date"));
            }
        }

        pub fn update_condns(&self, masked_condns: MaskedCondns) {
            self.buttons.update_condns(masked_condns);
            self.update_file_status_button();
        }

        pub fn update_session_needs_saving(&self, digest: &[u8]) {
            let mut condns: u64 = 0;
            let mask = Self::SAV_SESSION_NEEDS_SAVING;
            if digest != &self.current_file_digest.borrow()[..] {
                condns = Self::SAV_SESSION_NEEDS_SAVING;
            };
            self.buttons.update_condns(MaskedCondns { condns, mask });
            self.update_file_status_button();
        }

        pub fn tool_needs_saving(&self) -> bool {
            self.buttons.current_condns() & Self::SAV_TOOL_NEEDS_SAVING != 0
        }

        pub fn connect_write_to_file<F: Fn(&Path) -> Result<(), apaint::Error> + 'static>(
            &self,
            callback: F,
        ) {
            *self.write_file_callback.borrow_mut() = Some(Box::new(callback));
        }

        pub fn connect_read_from_file<F: Fn(&Path) -> Result<(), apaint::Error> + 'static>(
            &self,
            callback: F,
        ) {
            *self.load_file_callback.borrow_mut() = Some(Box::new(callback));
        }

        pub fn ok_to_reset(&self) -> bool {
            let status = self.buttons.current_condns();
            if status & (Self::SAV_SESSION_NEEDS_SAVING + Self::SAV_TOOL_NEEDS_SAVING) != 0 {
                if status & Self::SAV_IS_SAVEABLE != 0 {
                    let buttons = [
                        ("Cancel", gtk::ResponseType::Other(0)),
                        ("Save and Continue", gtk::ResponseType::Other(1)),
                        ("Continue Discarding Changes", gtk::ResponseType::Other(2)),
                    ];
                    match self.ask_question("There are unsaved changes!", None, &buttons) {
                        gtk::ResponseType::Other(0) => return false,
                        gtk::ResponseType::Other(1) => {
                            let temp = self.write_file_callback.borrow();
                            let write_callback = temp.as_ref().expect("programming error");
                            let o_path = self.current_file_path.borrow().clone();
                            if let Some(path) = o_path {
                                if let Err(err) = write_callback(&path) {
                                    self.report_error("Failed to save file", &err);
                                    return false;
                                }
                            } else if let Some(path) =
                                self.ask_file_path(Some("Save as: "), None, false)
                            {
                                if let Err(err) = write_callback(&path) {
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
    }
}
