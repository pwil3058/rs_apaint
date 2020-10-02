// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::rc::Rc;

use pw_gix::{
    gtk::{self, prelude::*, BoxExt, WidgetExt},
    wrapper::*,
};

use colour_math_gtk::{
    attributes::{ColourAttributeDisplayStack, ColourAttributeDisplayStackBuilder},
    manipulator::{ChromaLabel, RGBManipulatorGUI, RGBManipulatorGUIBuilder},
    rgb_entry::{RGBHexEntry, RGBHexEntryBuilder},
};

use crate::colour::*;

type ChangeCallback = Box<dyn Fn(RGB)>;

#[derive(PWO, Wrapper)]
pub struct ColourEditor {
    vbox: gtk::Box,
    rgb_manipulator: Rc<RGBManipulatorGUI>,
    cads: ColourAttributeDisplayStack,
    rgb_entry: Rc<RGBHexEntry<u16>>,
    change_callbacks: RefCell<Vec<ChangeCallback>>,
}

impl ColourEditor {
    pub fn new(scalar_attributes: &[ScalarAttribute], extra_buttons: &[gtk::Button]) -> Rc<Self> {
        let rgb_manipulator = RGBManipulatorGUIBuilder::new()
            .extra_buttons(extra_buttons)
            .chroma_label(if scalar_attributes.contains(&ScalarAttribute::Greyness) {
                if scalar_attributes.contains(&ScalarAttribute::Chroma) {
                    ChromaLabel::Both
                } else {
                    ChromaLabel::Greyness
                }
            } else {
                ChromaLabel::Chroma
            })
            .build();
        let rgb_entry = RGBHexEntryBuilder::new().editable(true).build();
        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(scalar_attributes)
            .build();
        let ced = Rc::new(Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            rgb_manipulator,
            cads,
            rgb_entry,
            change_callbacks: RefCell::new(Vec::new()),
        });

        ced.vbox.pack_start(&ced.cads.pwo(), false, false, 0);
        ced.vbox.pack_start(&ced.rgb_entry.pwo(), false, false, 0);
        ced.vbox
            .pack_start(&ced.rgb_manipulator.pwo(), true, true, 0);

        ced.vbox.show_all();

        let ced_c = Rc::clone(&ced);
        ced.rgb_entry
            .connect_value_changed(move |rgb| ced_c.set_rgb_and_inform(rgb.into()));

        let ced_c = Rc::clone(&ced);
        ced.rgb_manipulator
            .connect_changed(move |rgb| ced_c.set_rgb_and_inform(rgb));

        ced.reset();

        ced
    }
}

impl ColourEditor {
    pub fn set_rgb(&self, rgb: RGB) {
        self.rgb_entry.set_rgb(&rgb.into());
        self.rgb_manipulator.set_rgb(&rgb);
        self.cads.set_colour(Some(&rgb));
    }

    fn set_rgb_and_inform(&self, rgb: RGB) {
        self.set_rgb(rgb);
        for callback in self.change_callbacks.borrow().iter() {
            callback(rgb)
        }
    }

    pub fn reset(&self) {
        self.rgb_manipulator.reset();
        self.set_rgb_and_inform(RGB::WHITE * 0.5);
    }

    pub fn rgb(&self) -> RGB {
        self.rgb_manipulator.rgb()
    }

    pub fn connect_changed<F: Fn(RGB) + 'static>(&self, callback: F) {
        self.change_callbacks.borrow_mut().push(Box::new(callback))
    }
}
