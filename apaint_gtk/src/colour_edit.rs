// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::{BoxExt, ButtonExt, WidgetExt, WidgetExtManual};

use pw_gix::cairox::Draw;
use pw_gix::geometry::Point;
use pw_gix::gtkx::entry::{RGBEntryInterface, RGBHexEntryBox};
use pw_gix::wrapper::*;

use apaint_gtk_boilerplate::PWO;

use crate::angles::Degrees;
use crate::colour::*;
use pw_gix::gtkx::coloured::Colourable;

macro_rules! connect_button {
    ( $ed:ident, $btn:ident, $delta:ident, $apply:ident ) => {
        let ced_c = Rc::clone(&$ed);
        $ed.$btn.connect_clicked(move |btn| {
            let delta = ced_c.delta_size.get().$delta();
            let changed = ced_c.rgb_manipulator.borrow_mut().$apply(delta);
            if changed {
                let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                ced_c.set_rgb_and_inform(new_rgb);
            } else {
                btn.error_bell();
            }
        });
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum DeltaSize {
    Small,
    Normal,
    Large,
}

impl DeltaSize {
    fn for_value(&self) -> f64 {
        match *self {
            DeltaSize::Small => 0.0025,
            DeltaSize::Normal => 0.005,
            DeltaSize::Large => 0.01,
        }
    }

    fn for_chroma(&self) -> f64 {
        match *self {
            DeltaSize::Small => 0.0025,
            DeltaSize::Normal => 0.005,
            DeltaSize::Large => 0.01,
        }
    }

    fn for_hue_anticlockwise(&self) -> Degrees {
        match *self {
            DeltaSize::Small => 0.5.into(),
            DeltaSize::Normal => 1.0.into(),
            DeltaSize::Large => 5.0.into(),
        }
    }

    fn for_hue_clockwise(&self) -> Degrees {
        -self.for_hue_anticlockwise()
    }
}

struct Sample {
    pix_buf: gdk_pixbuf::Pixbuf,
    position: Point,
}

#[derive(PWO)]
pub struct ColourEditor {
    vbox: gtk::Box,
    rgb_manipulator: RefCell<RGBManipulator>,
    rgb_entry: RGBHexEntryBox,
    drawing_area: gtk::DrawingArea,
    incr_value_btn: gtk::Button,
    decr_value_btn: gtk::Button,
    hue_left_btn: gtk::Button,
    hue_right_btn: gtk::Button,
    decr_greyness_btn: gtk::Button,
    incr_greyness_btn: gtk::Button,
    decr_chroma_btn: gtk::Button,
    incr_chroma_btn: gtk::Button,
    delta_size: Cell<DeltaSize>,
}

impl ColourEditor {
    pub fn new() -> Rc<Self> {
        let ced = Rc::new(Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            rgb_manipulator: RefCell::new(RGBManipulator::new()),
            drawing_area: gtk::DrawingArea::new(),
            rgb_entry: RGBHexEntryBox::create(),
            incr_value_btn: gtk::Button::new_with_label("Value++"),
            decr_value_btn: gtk::Button::new_with_label("Value--"),
            hue_left_btn: gtk::Button::new_with_label("<"),
            hue_right_btn: gtk::Button::new_with_label(">"),
            decr_greyness_btn: gtk::Button::new_with_label("Greyness--"),
            incr_greyness_btn: gtk::Button::new_with_label("Greyness++"),
            decr_chroma_btn: gtk::Button::new_with_label("Chroma--"),
            incr_chroma_btn: gtk::Button::new_with_label("Chroma++"),
            delta_size: Cell::new(DeltaSize::Normal),
        });

        let events = gdk::EventMask::BUTTON_PRESS_MASK;
        ced.drawing_area.add_events(events);
        ced.drawing_area.set_size_request(200, 200);

        ced.vbox.pack_start(&ced.rgb_entry.pwo(), false, false, 0);
        ced.vbox.pack_start(&ced.incr_value_btn, false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&ced.hue_left_btn, false, false, 0);
        hbox.pack_start(&ced.drawing_area, true, true, 0);
        hbox.pack_start(&ced.hue_right_btn, false, false, 0);
        ced.vbox.pack_start(&hbox, true, true, 0);

        ced.vbox.pack_start(&ced.decr_value_btn, false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&ced.decr_chroma_btn, true, true, 0);
        hbox.pack_start(&ced.incr_chroma_btn, true, true, 0);
        ced.vbox.pack_start(&hbox, false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&ced.decr_greyness_btn, true, true, 0);
        hbox.pack_start(&ced.incr_greyness_btn, true, true, 0);
        ced.vbox.pack_start(&hbox, false, false, 0);

        ced.vbox.show_all();

        connect_button!(ced, incr_value_btn, for_value, incr_value);
        connect_button!(ced, decr_value_btn, for_value, decr_value);
        connect_button!(ced, incr_chroma_btn, for_chroma, incr_chroma);
        connect_button!(ced, decr_chroma_btn, for_chroma, decr_chroma);
        connect_button!(ced, incr_greyness_btn, for_chroma, decr_chroma);
        connect_button!(ced, decr_greyness_btn, for_chroma, incr_chroma);
        connect_button!(ced, hue_left_btn, for_hue_anticlockwise, rotate);
        connect_button!(ced, hue_right_btn, for_hue_clockwise, rotate);
        let events = gdk::EventMask::KEY_PRESS_MASK
            | gdk::EventMask::KEY_RELEASE_MASK
            | gdk::EventMask::ENTER_NOTIFY_MASK;
        ced.vbox.add_events(events);
        ced.vbox.set_receives_default(true);
        let ced_c = ced.clone();
        ced.vbox.connect_key_press_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::enums::key::Shift_L {
                ced_c.delta_size.set(DeltaSize::Large);
            } else if key == gdk::enums::key::Shift_R {
                ced_c.delta_size.set(DeltaSize::Small);
            };
            gtk::Inhibit(false)
        });
        let ced_c = ced.clone();
        ced.vbox.connect_key_release_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::enums::key::Shift_L || key == gdk::enums::key::Shift_R {
                ced_c.delta_size.set(DeltaSize::Normal);
            };
            gtk::Inhibit(false)
        });
        let ced_c = ced.clone();
        ced.vbox.connect_enter_notify_event(move |_, _| {
            ced_c.delta_size.set(DeltaSize::Normal);
            gtk::Inhibit(false)
        });

        let ced_c = Rc::clone(&ced);
        ced.drawing_area.connect_draw(move |_, cctx| {
            ced_c.draw(cctx);
            gtk::Inhibit(true)
        });

        let ced_c = Rc::clone(&ced);
        ced.rgb_entry
            .connect_value_changed(move |rgb| ced_c.set_rgb_and_inform(rgb));

        ced
    }
}

impl ColourEditor {
    fn set_rgb(&self, rgb: RGB) {
        self.rgb_entry.set_rgb(rgb);
        self.rgb_manipulator.borrow_mut().set_rgb(rgb);
        self.incr_value_btn
            .set_widget_colour_rgb(rgb * 0.8 + RGB::WHITE * 0.2);
        self.decr_value_btn.set_widget_colour_rgb(rgb * 0.8);
        if rgb.is_grey() {
            self.incr_greyness_btn.set_widget_colour_rgb(rgb);
            self.decr_greyness_btn.set_widget_colour_rgb(rgb);
            self.incr_chroma_btn.set_widget_colour_rgb(rgb);
            self.decr_chroma_btn.set_widget_colour_rgb(rgb);
            self.hue_left_btn.set_widget_colour_rgb(rgb);
            self.hue_right_btn.set_widget_colour_rgb(rgb);
        } else {
            let low_chroma_rgb = rgb * 0.8 + rgb.monotone_rgb() * 0.2;
            let high_chroma_rgb = rgb * 0.8 + rgb.max_chroma_rgb() * 0.2;
            self.incr_greyness_btn.set_widget_colour_rgb(low_chroma_rgb);
            self.decr_greyness_btn
                .set_widget_colour_rgb(high_chroma_rgb);
            self.incr_chroma_btn.set_widget_colour_rgb(high_chroma_rgb);
            self.decr_chroma_btn.set_widget_colour_rgb(low_chroma_rgb);

            self.hue_left_btn
                .set_widget_colour_rgb(rgb.components_rotated(Degrees::DEG_30));
            self.hue_right_btn
                .set_widget_colour_rgb(rgb.components_rotated(-Degrees::DEG_30));
        }
        self.drawing_area.queue_draw();
    }

    fn set_rgb_and_inform(&self, rgb: RGB) {
        self.set_rgb(rgb);
        // TODO: implement inform() component
    }

    fn draw(&self, cairo_context: &cairo::Context) {
        let rgb = self.rgb_manipulator.borrow().rgb();
        cairo_context.set_source_colour_rgb(rgb);
        cairo_context.paint();
    }
}
