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

    use apaint_gtk_boilerplate::PWO;

    use pw_gix::{
        gtkx::coloured::Colourable,
        sav_state::{
            ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled, SAV_NEXT_CONDN,
        },
        wrapper::*,
    };

    use crate::{colour::RGB, icon_image};

    #[derive(PWO)]
    pub struct MixerFileManager {
        hbox: gtk::Box,
        buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
        file_name_label: gtk::Label,
        file_status_btn: gtk::Button,
        current_file_path: RefCell<Option<PathBuf>>,
        current_file_digest: RefCell<Vec<u8>>,
    }

    impl MixerFileManager {
        const SAV_HAS_CURRENT_FILE: u64 = SAV_NEXT_CONDN << 0;
        const SAV_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 1;
        const SAV_MIX_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 2;
        const SAV_MIXES_NEED_SAVING: u64 = SAV_NEXT_CONDN << 3;
        const SAV_MIXES_ARE_SAVEABLE: u64 = SAV_NEXT_CONDN << 4;

        const BTN_IMAGE_SIZE: i32 = 24;

        pub fn new() -> Rc<Self> {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
                WidgetStatesControlled::Sensitivity,
                None,
                None,
            );

            let new_colln_btn = gtk::ButtonBuilder::new()
                .tooltip_text("Clear the mixer in preparation for creating a new mixing session")
                .build();
            // TODO: change setting of image when ButtonBuilder interface is fixed.
            new_colln_btn.set_image(Some(&icon_image::colln_new_image(Self::BTN_IMAGE_SIZE)));
            buttons.add_widget("new_colln", &new_colln_btn, 0);
            hbox.pack_start(&new_colln_btn, false, false, 0);

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
                Self::SAV_HAS_CURRENT_FILE + Self::SAV_MIXES_ARE_SAVEABLE,
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
                Self::SAV_MIXES_ARE_SAVEABLE,
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
                Self::SAV_MIXES_ARE_SAVEABLE + Self::SAV_MIXES_NEED_SAVING,
            );

            hbox.show_all();

            Rc::new(Self {
                hbox,
                buttons,
                file_name_label,
                file_status_btn,
                current_file_path: RefCell::new(None),
                current_file_digest: RefCell::new(vec![]),
            })
        }

        fn set_current_file_path<Q: AsRef<Path>>(&self, path: Option<Q>) {
            let mut condns: u64 = 0;
            let mask: u64 = Self::SAV_HAS_CURRENT_FILE;
            if let Some(path) = path {
                let path: PathBuf = path.as_ref().to_path_buf();
                self.file_name_label.set_label(&path.to_string_lossy());
                *self.current_file_path.borrow_mut() = Some(path);
                condns = Self::SAV_HAS_CURRENT_FILE;
            } else {
                *self.current_file_path.borrow_mut() = None;
                self.file_name_label.set_label("")
            }
            self.buttons.update_condns(MaskedCondns { condns, mask });
        }

        fn update_file_status_button(&self) {
            let current_condns = self.buttons.current_condns();
            if current_condns & Self::SAV_MIXES_NEED_SAVING != 0 {
                if current_condns & Self::SAV_MIXES_ARE_SAVEABLE != 0 {
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
    }
}
