// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::Cell, rc::Rc};

use gtk::prelude::*;

use apaint_gtk_boilerplate::{Wrapper, PWO};
use pw_gix::wrapper::*;

use apaint_cairo::*;

#[derive(PWO, Wrapper)]
pub struct Graticule {
    drawing_area: gtk::DrawingArea,
    graticule: apaint::graticule::Graticule<f64>,
    zoom: Cell<f64>,
    origin_offset: Cell<Point>,
    last_xy: Cell<Option<Point>>,
}

impl Graticule {
    pub fn new() -> Rc<Self> {
        let graticule = Rc::new(Self {
            drawing_area: gtk::DrawingArea::new(),
            graticule: apaint::graticule::Graticule::default(),
            origin_offset: Cell::new(Point::default()),
            zoom: Cell::new(1.0),
            last_xy: Cell::new(None),
        });
        graticule.drawing_area.set_size_request(200, 200);
        let events = gdk::EventMask::SCROLL_MASK
            | gdk::EventMask::BUTTON_PRESS_MASK
            | gdk::EventMask::BUTTON_MOTION_MASK
            | gdk::EventMask::LEAVE_NOTIFY_MASK
            | gdk::EventMask::BUTTON_RELEASE_MASK;
        graticule.drawing_area.add_events(events);

        let graticule_c = Rc::clone(&graticule);
        graticule
            .drawing_area
            .connect_draw(move |da, cairo_context| {
                let size: Size = Size {
                    width: da.get_allocated_width() as f64,
                    height: da.get_allocated_height() as f64,
                };
                let origin_offset = graticule_c.origin_offset.get();
                cairo_context.translate(origin_offset.x, origin_offset.y);
                let cartesian = CairoCartesian::new(cairo_context, size);
                cartesian.set_scale(graticule_c.zoom.get());
                graticule_c.graticule.draw(&cartesian);
                gtk::Inhibit(false)
            });

        // ZOOM
        let graticule_c = Rc::clone(&graticule);
        graticule
            .drawing_area
            .connect_scroll_event(move |da, scroll_event| {
                if let Some(device) = scroll_event.get_device() {
                    if device.get_source() == gdk::InputSource::Mouse {
                        match scroll_event.get_direction() {
                            gdk::ScrollDirection::Up => {
                                graticule_c.decr_zoom();
                                da.queue_draw();
                                return gtk::Inhibit(true);
                            }
                            gdk::ScrollDirection::Down => {
                                graticule_c.incr_zoom();
                                da.queue_draw();
                                return gtk::Inhibit(true);
                            }
                            _ => (),
                        }
                    }
                };
                gtk::Inhibit(false)
            });

        // MOVE ORIGIN
        let graticule_c = Rc::clone(&graticule);
        graticule
            .drawing_area
            .connect_button_press_event(move |_, event| {
                debug_assert_eq!(event.get_event_type(), gdk::EventType::ButtonPress);
                if event.get_button() == 1 {
                    graticule_c.last_xy.set(Some(event.get_position().into()));
                    return gtk::Inhibit(true);
                }
                Inhibit(false)
            });
        let graticule_c = Rc::clone(&graticule);
        graticule
            .drawing_area
            .connect_motion_notify_event(move |da, event| {
                if let Some(last_xy) = graticule_c.last_xy.get() {
                    let this_xy: Point = event.get_position().into();
                    let delta_xy = this_xy - last_xy;
                    graticule_c.last_xy.set(Some(this_xy));
                    graticule_c.shift_origin_offset(delta_xy);
                    da.queue_draw();
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            });
        let graticule_c = Rc::clone(&graticule);
        graticule
            .drawing_area
            .connect_button_release_event(move |_, event| {
                debug_assert_eq!(event.get_event_type(), gdk::EventType::ButtonRelease);
                if event.get_button() == 1 {
                    graticule_c.last_xy.set(None);
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            });
        let graticule_c = Rc::clone(&graticule);
        graticule
            .drawing_area
            .connect_leave_notify_event(move |_, _| {
                graticule_c.last_xy.set(None);
                gtk::Inhibit(false)
            });

        graticule
    }

    fn decr_zoom(&self) {
        let new_zoom = (self.zoom.get() - 0.025).max(1.0);
        self.zoom.set(new_zoom);
    }

    fn incr_zoom(&self) {
        let new_zoom = (self.zoom.get() + 0.025).min(10.0);
        self.zoom.set(new_zoom);
    }

    fn shift_origin_offset(&self, delta: Point) {
        let new_offset = self.origin_offset.get() + delta;
        self.origin_offset.set(new_offset);
    }
}
