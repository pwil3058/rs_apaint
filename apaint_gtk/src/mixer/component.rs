// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::prelude::*;

use apaint_gtk_boilerplate::PWO;
use pw_gix::{gtkx::coloured::Colourable, wrapper::*};

use apaint::{LabelText, TooltipText};

use crate::colour::{ColourInterface, RGB};

#[derive(PWO)]
pub struct PartsSpinButton<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Clone + 'static,
{
    event_box: gtk::EventBox,
    spin_button: gtk::SpinButton,
    paint: P,
    changed_callbacks: RefCell<Vec<Box<dyn Fn() + 'static>>>,
}

impl<P> PartsSpinButton<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Clone + 'static,
{
    pub fn new(paint: &P, sensitive: bool) -> Rc<Self> {
        let event_box = gtk::EventBoxBuilder::new()
            .tooltip_text(&paint.tooltip_text())
            .events(gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK)
            .build();
        let spin_button = gtk::SpinButtonBuilder::new()
            .adjustment(&gtk::Adjustment::new(0.0, 0.0, 999.0, 1.0, 10.0, 0.0))
            .climb_rate(0.0)
            .digits(0)
            .sensitive(sensitive)
            .numeric(true)
            .build();
        let label = gtk::Label::new(Some(&paint.label_text()));
        label.set_widget_colour_rgb(paint.rgb());
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&label, true, true, 0);
        hbox.pack_start(&spin_button, false, false, 0);
        let frame = gtk::FrameBuilder::new().build();
        frame.add(&hbox);
        event_box.add(&frame);
        let psb = Rc::new(Self {
            event_box,
            spin_button,
            paint: paint.clone(),
            changed_callbacks: RefCell::new(vec![]),
        });

        let psb_c = Rc::clone(&psb);
        psb.spin_button
            .connect_value_changed(move |_| psb_c.inform_changed());

        psb
    }

    fn parts(&self) -> u64 {
        self.spin_button.get_value_as_int() as u64
    }

    pub fn set_parts(&self, parts: u64) {
        self.spin_button.set_value(parts as f64);
    }

    pub fn rgb_parts(&self) -> (RGB, u64) {
        (self.paint.rgb(), self.parts())
    }

    pub fn connect_changed<F: Fn() + 'static>(&self, callback: F) {
        self.changed_callbacks.borrow_mut().push(Box::new(callback));
    }

    fn inform_changed(&self) {
        for callback in self.changed_callbacks.borrow().iter() {
            callback();
        }
    }
}

#[derive(PWO)]
pub struct PartsSpinButtonBox<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Clone + 'static,
{
    frame: gtk::Frame,
    vbox: gtk::Box,
    rows: RefCell<Vec<gtk::Box>>,
    spinners: RefCell<Vec<Rc<PartsSpinButton<P>>>>,
    sensitive: Cell<bool>,
    count: Cell<u32>,
    n_cols: Cell<u32>,
    contributions_changed_callbacks: RefCell<Vec<Box<dyn Fn() + 'static>>>,
}

impl<P> PartsSpinButtonBox<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Clone + 'static,
{
    pub fn new(title: &str, n_cols: u32, sensitive: bool) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let frame = gtk::FrameBuilder::new().label(title).build();
        frame.add(&vbox);
        Rc::new(Self {
            frame,
            vbox,
            rows: RefCell::new(vec![]),
            spinners: RefCell::new(vec![]),
            sensitive: Cell::new(sensitive),
            count: Cell::new(0),
            n_cols: Cell::new(n_cols),
            contributions_changed_callbacks: RefCell::new(vec![]),
        })
    }

    pub fn rgb_contributions(&self) -> Vec<(RGB, u64)> {
        let mut v = vec![];
        for spinner in self.spinners.borrow().iter() {
            let (rgb, parts) = spinner.rgb_parts();
            if parts > 0 {
                v.push((rgb, parts));
            }
        }
        v
    }

    fn pack_append<W: IsA<gtk::Widget>>(&self, widget: &W) {
        if self.count.get() % self.n_cols.get() == 0 {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            self.vbox.pack_start(&hbox, false, false, 0);
            self.rows.borrow_mut().push(hbox);
        };
        let last_index = self.rows.borrow().len() - 1;
        self.rows.borrow()[last_index].pack_start(widget, true, true, 0);
        self.count.set(self.count.get() + 1);
    }

    pub fn connect_contributions_changed<F: Fn() + 'static>(&self, callback: F) {
        self.contributions_changed_callbacks
            .borrow_mut()
            .push(Box::new(callback));
    }

    fn inform_contributions_changed(&self) {
        for callback in self.contributions_changed_callbacks.borrow().iter() {
            callback();
        }
    }
}

pub trait RcPartsSpinButtonBox<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Clone + 'static,
{
    fn add_paint(&self, paint: &P);
}

impl<P> RcPartsSpinButtonBox<P> for Rc<PartsSpinButtonBox<P>>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Clone + 'static,
{
    fn add_paint(&self, paint: &P) {
        let spinner = PartsSpinButton::new(paint, self.sensitive.get());
        self.pack_append(&spinner.pwo());
        let self_c = Rc::clone(self);
        spinner.connect_changed(move || self_c.inform_contributions_changed());
        self.spinners.borrow_mut().push(spinner);
        self.frame.show_all();
    }
}
