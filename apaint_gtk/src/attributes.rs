// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk::WidgetExt;

use std::cell::RefCell;
use std::rc::Rc;

use apaint_gtk_boilerplate::{Wrapper, PWO};
use pw_gix::wrapper::*;

use crate::colour::ColourInterface;
use crate::drawing::Drawer;
use apaint::attributes::ColourAttributeDisplayIfce;
use apaint::drawing::Size;

#[derive(PWO, Wrapper)]
pub struct ColourAttributeDisplay<A: ColourAttributeDisplayIfce<f64> + 'static> {
    drawing_area: gtk::DrawingArea,
    attribute: RefCell<A>,
}

impl<A: ColourAttributeDisplayIfce<f64> + 'static> ColourAttributeDisplay<A> {
    pub fn new() -> Rc<Self> {
        let cad = Rc::new(Self {
            drawing_area: gtk::DrawingArea::new(),
            attribute: RefCell::new(A::new()),
        });
        cad.drawing_area.set_size_request(90, 30);
        let cad_c = Rc::clone(&cad);
        cad.drawing_area.connect_draw(move |da, cairo_context| {
            let size: Size<f64> = Size {
                width: da.get_allocated_width() as f64,
                height: da.get_allocated_height() as f64,
            };
            let drawer = Drawer::new(cairo_context, size);
            cad_c.attribute.borrow().draw_all(&drawer);
            gtk::Inhibit(false)
        });
        cad
    }

    pub fn set_colour(&self, colour: Option<impl ColourInterface<f64>>) {
        self.attribute.borrow_mut().set_colour(colour);
    }

    pub fn set_target_colour(&self, colour: Option<impl ColourInterface<f64>>) {
        self.attribute.borrow_mut().set_target_colour(colour);
    }
}
