// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{process::Command, rc::Rc};

use gtk::prelude::*;

use pw_gix::wrapper::*;

#[derive(PWO, Wrapper)]
pub struct ModellersColourMixerMatcherTK {
    vbox: gtk::Box,
}

impl ModellersColourMixerMatcherTK {
    pub fn new() -> Rc<Self> {
        let mcmmtk = Rc::new(Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
        });
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        mcmmtk.vbox.pack_start(&hbox, false, false, 0);
        let button = gtk::Button::new_with_label("PDF Viewer");
        hbox.pack_start(&button, false, false, 0);
        let mcmmtk_c = Rc::clone(&mcmmtk);
        button.connect_clicked(move |_| mcmmtk_c.launch_pdf_viewer());

        mcmmtk.vbox.show_all();

        mcmmtk
    }

    pub fn ok_to_quit(&self) -> bool {
        self.ask_confirm_action("OK to quit?", None)
    }

    fn launch_pdf_viewer(&self) {
        // TODO: make pdf viewer configurable
        let viewer = "xreader";
        if let Err(err) = Command::new(viewer).spawn() {
            let msg = format!("Error running \"{}\"", viewer);
            self.report_error(&msg, &err);
        }
    }
}
