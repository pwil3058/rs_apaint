// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk::{BoxExt, RadioButtonExt, ToggleButtonExt, WidgetExt};

use std::cell::RefCell;
use std::rc::Rc;

use pw_gix::wrapper::*;

use crate::colour::{ColourInterface, ScalarAttribute, RGB};
use apaint::attributes::{
    ChromaCAD, ColourAttributeDisplayIfce, GreynessCAD, HueCAD, ValueCAD, WarmthCAD,
};
use apaint_cairo::{Drawer, Size};

pub trait DynColourAttributeDisplay: PackableWidgetObject<PWT = gtk::DrawingArea> {
    fn set_rgb(&self, rgb: Option<&RGB>);
    fn set_target_rgb(&self, rgb: Option<&RGB>);
}

#[derive(PWO, Wrapper)]
pub struct ColourAttributeDisplayStack {
    vbox: gtk::Box,
    cads: Vec<Rc<dyn DynColourAttributeDisplay>>,
}

impl ColourAttributeDisplayStack {
    pub fn new_obsolete(cads: &[Rc<dyn DynColourAttributeDisplay>]) -> Self {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        for cad in cads.iter() {
            vbox.pack_start(&cad.pwo(), true, true, 0);
        }
        Self {
            vbox,
            cads: cads.to_vec(),
        }
    }

    pub fn new(scalar_attributes: &[ScalarAttribute]) -> Self {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let mut cads = vec![];
        let hue_cad: Rc<dyn DynColourAttributeDisplay> =
            ColourAttributeDisplay::<HueCAD<f64>>::new();
        vbox.pack_start(&hue_cad.pwo(), true, true, 0);
        cads.push(hue_cad);
        for scalar_attribute in scalar_attributes.iter() {
            let cad: Rc<dyn DynColourAttributeDisplay> = match scalar_attribute {
                ScalarAttribute::Value => ColourAttributeDisplay::<ValueCAD<f64>>::new(),
                ScalarAttribute::Chroma => ColourAttributeDisplay::<ChromaCAD<f64>>::new(),
                ScalarAttribute::Warmth => ColourAttributeDisplay::<WarmthCAD<f64>>::new(),
                ScalarAttribute::Greyness => ColourAttributeDisplay::<GreynessCAD<f64>>::new(),
            };
            vbox.pack_start(&cad.pwo(), true, true, 0);
            cads.push(cad);
        }
        Self { vbox, cads }
    }

    pub fn set_colour(&self, colour: Option<&impl ColourInterface<f64>>) {
        for cad in self.cads.iter() {
            if let Some(colour) = colour {
                cad.set_rgb(Some(&colour.rgb()));
            } else {
                cad.set_rgb(None);
            }
        }
    }

    pub fn set_target_colour(&self, colour: Option<&impl ColourInterface<f64>>) {
        for cad in self.cads.iter() {
            if let Some(colour) = colour {
                cad.set_target_rgb(Some(&colour.rgb()));
            } else {
                cad.set_target_rgb(None);
            }
        }
    }
}

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
            let size: Size = Size {
                width: da.get_allocated_width() as f64,
                height: da.get_allocated_height() as f64,
            };
            let drawer = Drawer::new(cairo_context, size);
            cad_c.attribute.borrow().draw_all(&drawer);
            gtk::Inhibit(false)
        });
        cad
    }
}

impl<A> DynColourAttributeDisplay for ColourAttributeDisplay<A>
where
    A: ColourAttributeDisplayIfce<f64> + 'static,
{
    fn set_rgb(&self, rgb: Option<&RGB>) {
        self.attribute.borrow_mut().set_colour(rgb);
        self.drawing_area.queue_draw();
    }

    fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.attribute.borrow_mut().set_target_colour(rgb);
        self.drawing_area.queue_draw();
    }
}

type SelectionCallback = Box<dyn Fn(ScalarAttribute)>;

#[derive(PWO)]
pub struct AttributeSelectorRadioButtons {
    gtk_box: gtk::Box,
    callbacks: RefCell<Vec<SelectionCallback>>,
}

impl AttributeSelectorRadioButtons {
    pub fn new(orientation: gtk::Orientation, attributes: &[ScalarAttribute]) -> Rc<Self> {
        let asrb = Rc::new(Self {
            gtk_box: gtk::Box::new(orientation, 0),
            callbacks: RefCell::new(vec![]),
        });

        let mut first: Option<gtk::RadioButton> = None;
        for attr in attributes.iter() {
            let button = gtk::RadioButton::new_with_label(&attr.to_string());
            asrb.gtk_box.pack_start(&button, false, false, 0);
            if let Some(ref first) = first {
                button.join_group(Some(first))
            } else {
                first = Some(button.clone())
            }
            let asrb_c = Rc::clone(&asrb);
            let attr = *attr;
            button.connect_toggled(move |button| {
                let its_us = button.get_active();
                if its_us {
                    asrb_c.notify(attr);
                }
            });
        }

        asrb
    }

    pub fn connect_changed<F: Fn(ScalarAttribute) + 'static>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Box::new(callback))
    }

    fn notify(&self, attr: ScalarAttribute) {
        for callback in self.callbacks.borrow().iter() {
            callback(attr);
        }
    }
}
