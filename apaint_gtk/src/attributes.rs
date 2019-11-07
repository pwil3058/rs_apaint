// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk::{BoxExt, WidgetExt};

use std::cell::RefCell;
use std::rc::Rc;

use apaint_gtk_boilerplate::{Wrapper, PWO};
use pw_gix::wrapper::*;

use crate::colour::ColourInterface;
use crate::drawing::Drawer;
use apaint::attributes::{ColourAttributeDisplayIfce, HueCAD, ValueCAD, WarmthCAD};
use apaint::drawing::Size;
use colour_math::ScalarAttribute;

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
        self.drawing_area.queue_draw();
    }

    pub fn set_target_colour(&self, colour: Option<impl ColourInterface<f64>>) {
        self.attribute.borrow_mut().set_target_colour(colour);
        self.drawing_area.queue_draw();
    }
}

pub trait CADStackIfce: PackableWidgetObject {
    fn attributes() -> Vec<ScalarAttribute>;

    fn set_colour(&self, colour: Option<impl ColourInterface<f64>>);
    fn set_target_colour(&self, colour: Option<impl ColourInterface<f64>>);
}

#[derive(PWO, Wrapper)]
pub struct ArtistCADS {
    vbox: gtk::Box,
    hue_cad: Rc<ColourAttributeDisplay<HueCAD<f64>>>,
    value_cad: Rc<ColourAttributeDisplay<ValueCAD<f64>>>,
    warmth_cad: Rc<ColourAttributeDisplay<WarmthCAD<f64>>>,
}

impl ArtistCADS {
    pub fn new() -> Self {
        let acads = Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            hue_cad: ColourAttributeDisplay::<HueCAD<f64>>::new(),
            value_cad: ColourAttributeDisplay::<ValueCAD<f64>>::new(),
            warmth_cad: ColourAttributeDisplay::<WarmthCAD<f64>>::new(),
        };
        acads.vbox.pack_start(&acads.hue_cad.pwo(), true, true, 0);
        acads.vbox.pack_start(&acads.value_cad.pwo(), true, true, 0);
        acads
            .vbox
            .pack_start(&acads.warmth_cad.pwo(), true, true, 0);
        acads.vbox.show_all();
        acads
    }
}

impl CADStackIfce for ArtistCADS {
    fn attributes() -> Vec<ScalarAttribute> {
        vec![]
    }

    fn set_colour(&self, colour: Option<impl ColourInterface<f64>>) {
        self.hue_cad.set_colour(colour);
        self.value_cad.set_colour(colour);
        self.warmth_cad.set_colour(colour);
    }

    fn set_target_colour(&self, colour: Option<impl ColourInterface<f64>>) {
        self.hue_cad.set_target_colour(colour);
        self.value_cad.set_target_colour(colour);
        self.warmth_cad.set_target_colour(colour);
    }
}
