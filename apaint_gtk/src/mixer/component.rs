// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gcd::Gcd;

use pw_gix::{
    gdk,
    gtk::{self, prelude::*, ContainerExt, WidgetExt},
    gtkx::menu::WrappedMenu,
    wrapper::*,
};

use apaint::{LabelText, TooltipText};

use crate::colour::{ColourInterface, Colourable, RGB};

//type RemovalCallback<P: ColourInterface<f64> + TooltipText + LabelText + Ord + 'static> =
//    Box<dyn Fn(&Rc<P>)>;

type RemoveCallback<P> = Box<dyn Fn(&Rc<P>)>;

#[derive(PWO)]
pub struct PartsSpinButton<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Ord + 'static,
{
    event_box: gtk::EventBox,
    spin_button: gtk::SpinButton,
    popup_menu: WrappedMenu,
    paint: Rc<P>,
    changed_callbacks: RefCell<Vec<Box<dyn Fn() + 'static>>>,
    remove_me_callbacks: RefCell<Vec<RemoveCallback<P>>>,
}

impl<P> PartsSpinButton<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Ord + 'static,
{
    pub fn new(paint: &Rc<P>, sensitive: bool) -> Rc<Self> {
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
        label.set_widget_colour_rgb(&paint.rgb());
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&label, true, true, 0);
        hbox.pack_start(&spin_button, false, false, 0);
        let frame = gtk::FrameBuilder::new().build();
        frame.add(&hbox);
        event_box.add(&frame);
        let psb = Rc::new(Self {
            event_box,
            spin_button,
            popup_menu: WrappedMenu::new(&[]),
            paint: Rc::clone(paint),
            changed_callbacks: RefCell::new(vec![]),
            remove_me_callbacks: RefCell::new(vec![]),
        });

        let psb_c = Rc::clone(&psb);
        psb.spin_button
            .connect_value_changed(move |_| psb_c.inform_changed());

        let psb_c = Rc::clone(&psb);
        psb.popup_menu
            .append_item(
                "remove",
                "Remove Me",
                "Remove this paint form the palette/mixer",
            )
            .connect_activate(move |_| psb_c.inform_remove_me());

        let psb_c = Rc::clone(&psb);
        psb.event_box.connect_button_press_event(move |_, event| {
            if event.get_event_type() == gdk::EventType::ButtonPress && event.get_button() == 3 {
                psb_c
                    .popup_menu
                    .set_sensitivities(psb_c.parts() == 0, &["remove"]);
                psb_c.popup_menu.popup_at_event(event);
                return Inhibit(true);
            };
            gtk::Inhibit(false)
        });

        psb
    }

    fn parts(&self) -> u64 {
        self.spin_button.get_value_as_int() as u64
    }

    pub fn set_parts(&self, parts: u64) {
        self.spin_button.set_value(parts as f64);
    }

    pub fn divide_parts_by(&self, divisor: u64) {
        self.set_parts(self.parts() / divisor);
    }

    pub fn rgb_parts(&self) -> (RGB, u64) {
        (self.paint.rgb(), self.parts())
    }

    pub fn paint_parts(&self) -> (&Rc<P>, u64) {
        (&self.paint, self.parts())
    }

    pub fn connect_changed<F: Fn() + 'static>(&self, callback: F) {
        self.changed_callbacks.borrow_mut().push(Box::new(callback));
    }

    fn inform_changed(&self) {
        for callback in self.changed_callbacks.borrow().iter() {
            callback();
        }
    }

    pub fn connect_remove_me<F: Fn(&Rc<P>) + 'static>(&self, callback: F) {
        self.remove_me_callbacks
            .borrow_mut()
            .push(Box::new(callback));
    }

    fn inform_remove_me(&self) {
        for callback in self.remove_me_callbacks.borrow().iter() {
            callback(&self.paint)
        }
    }
}

#[derive(PWO)]
pub struct PartsSpinButtonBox<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Ord + 'static,
{
    frame: gtk::Frame,
    vbox: gtk::Box,
    spinners: RefCell<Vec<Rc<PartsSpinButton<P>>>>,
    sensitive: Cell<bool>,
    n_cols: Cell<u32>,
    contributions_changed_callbacks: RefCell<Vec<Box<dyn Fn() + 'static>>>,
    removal_requested_callbacks: RefCell<Vec<RemoveCallback<P>>>,
}

impl<P> PartsSpinButtonBox<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Ord + 'static,
{
    pub fn new(title: &str, n_cols: u32, sensitive: bool) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let frame = gtk::FrameBuilder::new().label(title).build();
        frame.add(&vbox);
        Rc::new(Self {
            frame,
            vbox,
            spinners: RefCell::new(vec![]),
            sensitive: Cell::new(sensitive),
            n_cols: Cell::new(n_cols),
            contributions_changed_callbacks: RefCell::new(vec![]),
            removal_requested_callbacks: RefCell::new(vec![]),
        })
    }

    pub fn rgb_contributions(&self) -> Vec<(RGB, u64)> {
        self.spinners
            .borrow()
            .iter()
            .map(|s| s.rgb_parts())
            .collect()
    }

    pub fn paint_contributions(&self) -> Vec<(Rc<P>, u64)> {
        self.spinners
            .borrow()
            .iter()
            .filter_map(|s| {
                let (paint, parts) = s.paint_parts();
                if parts > 0 {
                    Some((Rc::clone(paint), parts))
                } else {
                    None
                }
            })
            .collect()
    }

    fn binary_search_paint(&self, paint: &P) -> Result<usize, usize> {
        self.spinners
            .borrow()
            .binary_search_by_key(&paint, |s| &s.paint)
    }

    fn repack_all(&self) {
        for row_widget in self.vbox.get_children() {
            let row = row_widget.downcast::<gtk::Box>().unwrap();
            for child in row.get_children() {
                row.remove(&child)
            }
            self.vbox.remove(&row);
        }
        if !self.spinners.borrow().is_empty() {
            let mut current_row = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            self.vbox.pack_start(&current_row, false, false, 0);
            for (count, spinner) in self.spinners.borrow().iter().enumerate() {
                if count > 0 && count % self.n_cols.get() as usize == 0 {
                    current_row = gtk::Box::new(gtk::Orientation::Horizontal, 1);
                    self.vbox.pack_start(&current_row, false, false, 0);
                }
                current_row.pack_start(&spinner.pwo(), true, true, 0);
            }
        };
        self.frame.show_all()
    }

    pub fn remove_paint(&self, paint: &Rc<P>) {
        if let Ok(index) = self.binary_search_paint(paint) {
            let spinner = self.spinners.borrow_mut().remove(index);
            self.repack_all();
            if spinner.parts() > 0 {
                self.inform_contributions_changed();
            }
        }
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

    pub fn connect_removal_requested<F: Fn(&Rc<P>) + 'static>(&self, callback: F) {
        self.removal_requested_callbacks
            .borrow_mut()
            .push(Box::new(callback));
    }

    fn inform_removal_requested(&self, paint: &Rc<P>) {
        for callback in self.removal_requested_callbacks.borrow().iter() {
            callback(paint);
        }
    }

    pub fn parts_gcd(&self) -> u64 {
        self.spinners
            .borrow()
            .iter()
            .fold(0, |gcd, s| gcd.gcd(s.parts()))
    }

    pub fn zero_all_parts(&self) {
        for spinner in self.spinners.borrow().iter() {
            spinner.set_parts(0);
        }
    }

    pub fn div_all_parts_by(&self, divisor: u64) {
        if divisor > 0 {
            for spinner in self.spinners.borrow().iter() {
                spinner.divide_parts_by(divisor);
            }
        }
    }

    // TODO: is has_contributions() really needed
    pub fn has_contributions(&self) -> bool {
        self.spinners.borrow().iter().any(|s| s.parts() > 0)
    }
}

pub trait RcPartsSpinButtonBox<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Ord + 'static,
{
    fn add_paint(&self, paint: &Rc<P>);
}

impl<P> RcPartsSpinButtonBox<P> for Rc<PartsSpinButtonBox<P>>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Ord + 'static,
{
    fn add_paint(&self, paint: &Rc<P>) {
        if let Err(index) = self.binary_search_paint(paint) {
            let spinner = PartsSpinButton::new(paint, self.sensitive.get());
            let self_c = Rc::clone(self);
            spinner.connect_changed(move || self_c.inform_contributions_changed());
            let self_c = Rc::clone(self);
            spinner.connect_remove_me(move |paint| self_c.inform_removal_requested(paint));
            self.spinners.borrow_mut().insert(index, spinner);
            self.repack_all();
            self.frame.show_all();
        } else {
            // quietly ignore request to add a paint already in the mixer
        }
    }
}
