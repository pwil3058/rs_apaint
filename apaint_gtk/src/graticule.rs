// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::prelude::*;

use apaint_gtk_boilerplate::{Wrapper, PWO};
use pw_gix::{gtkx::menu::ManagedMenu, wrapper::*};

use apaint_cairo::*;
use pw_gix::sav_state::{MaskedCondns, WidgetStatesControlled};

#[derive(PWO, Wrapper)]
pub struct GtkGraticule<G, T>
where
    G: apaint::graticule::Graticule<f64, T> + Sized,
{
    drawing_area: gtk::DrawingArea,
    graticule: G,
    chosen_item: RefCell<Option<T>>,
    popup_menu: ManagedMenu,
    zoom: Cell<f64>,
    origin_offset: Cell<Point>,
    last_xy: Cell<Option<Point>>,
}

impl<G, T> GtkGraticule<G, T>
where
    G: apaint::graticule::Graticule<f64, T> + Sized + Default + 'static,
    T: 'static,
{
    const HAS_CHOSEN_ITEM: u64 = 1;

    pub fn new(menu_items: &[(&str, &str, Option<&gtk::Image>, &str, u64)]) -> Rc<Self> {
        let popup_menu =
            ManagedMenu::new(WidgetStatesControlled::Sensitivity, None, None, menu_items);
        let gtk_graticule = Rc::new(Self {
            drawing_area: gtk::DrawingArea::new(),
            graticule: G::default(),
            chosen_item: RefCell::new(None),
            popup_menu,
            origin_offset: Cell::new(Point::default()),
            zoom: Cell::new(1.0),
            last_xy: Cell::new(None),
        });
        gtk_graticule.drawing_area.set_size_request(200, 200);
        gtk_graticule.drawing_area.set_has_tooltip(true);
        let events = gdk::EventMask::SCROLL_MASK
            | gdk::EventMask::BUTTON_PRESS_MASK
            | gdk::EventMask::BUTTON_MOTION_MASK
            | gdk::EventMask::LEAVE_NOTIFY_MASK
            | gdk::EventMask::BUTTON_RELEASE_MASK;
        gtk_graticule.drawing_area.add_events(events);

        let gtk_graticule_c = Rc::clone(&gtk_graticule);
        gtk_graticule
            .drawing_area
            .connect_draw(move |_, cairo_context| {
                cairo_context.transform(gtk_graticule_c.current_transform_matrix());
                gtk_graticule_c
                    .graticule
                    .draw(&CairoCartesian::new(cairo_context));
                gtk::Inhibit(false)
            });

        // ZOOM
        let gtk_graticule_c = Rc::clone(&gtk_graticule);
        gtk_graticule
            .drawing_area
            .connect_scroll_event(move |da, scroll_event| {
                if let Some(device) = scroll_event.get_device() {
                    if device.get_source() == gdk::InputSource::Mouse {
                        match scroll_event.get_direction() {
                            gdk::ScrollDirection::Up => {
                                gtk_graticule_c.decr_zoom();
                                da.queue_draw();
                                return gtk::Inhibit(true);
                            }
                            gdk::ScrollDirection::Down => {
                                gtk_graticule_c.incr_zoom();
                                da.queue_draw();
                                return gtk::Inhibit(true);
                            }
                            _ => (),
                        }
                    }
                };
                gtk::Inhibit(false)
            });

        // COMMENCE MOVE ORIGIN OR POPUP MENU
        let gtk_graticule_c = Rc::clone(&gtk_graticule);
        gtk_graticule
            .drawing_area
            .connect_button_press_event(move |_, event| {
                debug_assert_eq!(event.get_event_type(), gdk::EventType::ButtonPress);
                match event.get_button() {
                    1 => {
                        gtk_graticule_c
                            .last_xy
                            .set(Some(event.get_position().into()));
                        gtk::Inhibit(true)
                    }
                    3 => {
                        if let Some(item) = gtk_graticule_c
                            .graticule
                            .item_at_point(event.get_position().into())
                        {
                            *gtk_graticule_c.chosen_item.borrow_mut() = Some(item);
                            gtk_graticule_c.popup_menu.update_condns(MaskedCondns {
                                condns: Self::HAS_CHOSEN_ITEM,
                                mask: Self::HAS_CHOSEN_ITEM,
                            });
                        } else {
                            *gtk_graticule_c.chosen_item.borrow_mut() = None;
                            gtk_graticule_c.popup_menu.update_condns(MaskedCondns {
                                condns: 0,
                                mask: Self::HAS_CHOSEN_ITEM,
                            });
                        };
                        gtk_graticule_c.popup_menu.popup_at_event(event);
                        gtk::Inhibit(true)
                    }
                    _ => gtk::Inhibit(false),
                }
            });

        // MOVE ORIGIN
        let gtk_graticule_c = Rc::clone(&gtk_graticule);
        gtk_graticule
            .drawing_area
            .connect_motion_notify_event(move |da, event| {
                if let Some(last_xy) = gtk_graticule_c.last_xy.get() {
                    let this_xy: Point = event.get_position().into();
                    let delta_xy = this_xy - last_xy;
                    gtk_graticule_c.last_xy.set(Some(this_xy));
                    gtk_graticule_c.shift_origin_offset(delta_xy);
                    da.queue_draw();
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            });
        let gtk_graticule_c = Rc::clone(&gtk_graticule);
        gtk_graticule
            .drawing_area
            .connect_button_release_event(move |_, event| {
                debug_assert_eq!(event.get_event_type(), gdk::EventType::ButtonRelease);
                if event.get_button() == 1 {
                    gtk_graticule_c.last_xy.set(None);
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            });
        let gtk_graticule_c = Rc::clone(&gtk_graticule);
        gtk_graticule
            .drawing_area
            .connect_leave_notify_event(move |_, _| {
                gtk_graticule_c.last_xy.set(None);
                gtk::Inhibit(false)
            });

        // TOOLTIP
        let gtk_graticule_c = Rc::clone(&gtk_graticule);
        gtk_graticule
            .drawing_area
            .connect_query_tooltip(move |_, x, y, _, tooltip| {
                let point = gtk_graticule_c.device_to_user(x as f64, y as f64);
                if let Some(text) = gtk_graticule_c.graticule.tooltip_for_point(point) {
                    tooltip.set_text(Some(&text));
                    true
                } else {
                    false
                }
            });

        gtk_graticule
    }

    fn decr_zoom(&self) {
        let new_zoom = (self.zoom.get() - 0.025).max(1.0);
        self.zoom.set(new_zoom);
    }

    fn incr_zoom(&self) {
        let new_zoom = (self.zoom.get() + 0.025).min(10.0);
        self.zoom.set(new_zoom);
    }

    fn current_transform_matrix(&self) -> cairo::Matrix {
        let zoom = self.zoom.get();
        let origin_offset = self.origin_offset.get();
        let mut ctm = CairoCartesian::cartesian_transform_matrix(
            self.drawing_area.get_allocated_width() as f64,
            self.drawing_area.get_allocated_height() as f64,
        );
        ctm.scale(zoom, zoom);
        ctm.translate(origin_offset.x, origin_offset.y);
        ctm
    }

    fn device_to_user(&self, x: f64, y: f64) -> Point {
        let mut ctm = self.current_transform_matrix();
        ctm.invert();
        ctm.transform_point(x, y).into()
    }

    fn device_to_user_delta(&self, point: Point) -> Point {
        let mut ctm = self.current_transform_matrix();
        ctm.invert();
        ctm.transform_distance(point.x, point.y).into()
    }

    fn shift_origin_offset(&self, device_delta: Point) {
        let delta = self.device_to_user_delta(device_delta);
        self.origin_offset.set(self.origin_offset.get() + delta);
    }
}
