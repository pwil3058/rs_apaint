// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use pw_gix::wrapper::*;

use colour_math::ScalarAttribute;

use apaint_gtk_boilerplate::PWO;

use crate::attributes::ColourAttributeDisplayStack;

#[derive(PWO)]
pub struct TargetedPaintEntry {
    vbox: gtk::Box,
    id_label: gtk::Label,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    cads: ColourAttributeDisplayStack,
    drawing_area: gtk::DrawingArea,
}

impl TargetedPaintEntry {
    pub fn new(attributes: &[ScalarAttribute]) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let id_label = gtk::LabelBuilder::new().label("#???").build();
        let name_entry = gtk::EntryBuilder::new().build();
        let notes_entry = gtk::EntryBuilder::new().build();
        let cads = ColourAttributeDisplayStack::new(attributes);
        let drawing_area = gtk::DrawingAreaBuilder::new().height_request(100).build();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&id_label, false, false, 0);
        hbox.pack_start(&name_entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&gtk::Label::new(Some("Notes: ")), false, false, 0);
        hbox.pack_start(&notes_entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&cads.pwo(), false, false, 0);
        vbox.pack_start(&drawing_area, true, true, 0);
        Rc::new(Self {
            vbox,
            id_label,
            name_entry,
            notes_entry,
            cads,
            drawing_area,
        })
    }
}
