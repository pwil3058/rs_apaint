// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use pw_gix::wrapper::*;

#[derive(PWO, Wrapper)]
pub struct ModellersColourMixerMatcherTK {
    vbox: gtk::Box,
}

impl ModellersColourMixerMatcherTK {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
        })
    }

    pub fn ok_to_quit(&self) -> bool {
        self.ask_confirm_action("OK to quit?", None)
    }
}
