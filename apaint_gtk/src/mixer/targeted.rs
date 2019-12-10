// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;

use cairo;

use pw_gix::{cairox::*, wrapper::*};

use colour_math::ScalarAttribute;

use apaint_gtk_boilerplate::PWO;

use crate::{attributes::ColourAttributeDisplayStack, colour::RGB};

#[derive(PWO)]
pub struct TargetedPaintEntry {
    vbox: gtk::Box,
    id_label: gtk::Label,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    cads: ColourAttributeDisplayStack,
    drawing_area: gtk::DrawingArea,
    mix_rgb: RefCell<Option<RGB>>,
    target_rgb: RefCell<Option<RGB>>,
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
        vbox.show_all();
        let tpe = Rc::new(Self {
            vbox,
            id_label,
            name_entry,
            notes_entry,
            cads,
            drawing_area,
            mix_rgb: RefCell::new(None),
            target_rgb: RefCell::new(None),
        });

        let tpe_c = Rc::clone(&tpe);
        tpe.drawing_area.connect_draw(move |da, ctxt| {
            tpe_c.draw(da, ctxt);
            Inhibit(false)
        });

        tpe
    }

    pub fn draw(&self, drawing_area: &gtk::DrawingArea, cairo_context: &cairo::Context) {
        if let Some(ref rgb) = *self.mix_rgb.borrow() {
            cairo_context.set_source_colour_rgb(*rgb);
        } else {
            cairo_context.set_source_colour_rgb(RGB::BLACK);
        };
        cairo_context.paint();
        if let Some(ref rgb) = *self.target_rgb.borrow() {
            cairo_context.set_source_colour_rgb(*rgb);
            let width = drawing_area.get_allocated_width() as f64;
            let height = drawing_area.get_allocated_height() as f64;
            cairo_context.rectangle(width / 4.0, height / 4.0, width / 2.0, height / 2.0);
            cairo_context.fill();
        }
    }

    pub fn set_mix_rgb(&self, rgb: Option<&RGB>) {
        if let Some(rgb) = rgb {
            *self.mix_rgb.borrow_mut() = Some(*rgb);
            self.cads.set_colour(Some(rgb));
        } else {
            *self.mix_rgb.borrow_mut() = None;
            self.cads.set_colour(Option::<&RGB>::None);
        }
        self.drawing_area.queue_draw()
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        // TODO: find out why target color doesn't show up in cads
        if let Some(rgb) = rgb {
            *self.target_rgb.borrow_mut() = Some(*rgb);
            self.cads.set_target_colour(Some(rgb));
        } else {
            *self.target_rgb.borrow_mut() = None;
            self.cads.set_target_colour(Option::<&RGB>::None);
        }
        self.drawing_area.queue_draw()
    }
}
