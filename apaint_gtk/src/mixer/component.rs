// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use apaint_gtk_boilerplate::PWO;
use pw_gix::{gtkx::coloured::Colourable, wrapper::*};

use apaint::{BasicPaintIfce, LabelText, TooltipText};
use colour_math::ColourInterface;

#[derive(PWO)]
pub struct PartsSpinButton<P>
where
    P: ColourInterface<f64>,
{
    event_box: gtk::EventBox,
    spin_button: gtk::SpinButton,
    paint: P,
}

impl<P> PartsSpinButton<P>
where
    P: ColourInterface<f64> + TooltipText + LabelText + Clone,
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
        Rc::new(Self {
            event_box,
            spin_button,
            paint: paint.clone(),
        })
    }
}
