// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use apaint_gtk_boilerplate::{Wrapper, PWO};
use pw_gix::wrapper::*;

use apaint_cairo::*;

#[derive(PWO, Wrapper)]
pub struct Graticule {
    drawing_area: gtk::DrawingArea,
}

impl Graticule {
    pub fn new() -> Rc<Self> {
        let cad = Rc::new(Self {
            drawing_area: gtk::DrawingArea::new(),
        });
        cad.drawing_area.set_size_request(200, 200);
        let cad_c = Rc::clone(&cad);
        cad.drawing_area.connect_draw(move |da, cairo_context| {
            let size: Size = Size {
                width: da.get_allocated_width() as f64,
                height: da.get_allocated_height() as f64,
            };
            let cartesian = CairoCartesian::new(cairo_context, size);
            cad_c.draw(&cartesian);
            gtk::Inhibit(false)
        });
        cad
    }

    fn draw(&self, cartesian: &CairoCartesian) {
        cartesian.set_line_width(0.01);
        cartesian.draw_line(&[Point { x: -1.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }]);
        cartesian.draw_line(&[Point { x: 0.0, y: 1.0 }, Point { x: 0.0, y: -1.0 }]);
        cartesian.draw_circle(Point { x: 0.0, y: 0.0 }, 1.0, false);
        cartesian.draw_diamond(Point { x: 0.5, y: 0.5 }, 0.1, false);
    }
}
