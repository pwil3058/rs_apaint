// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::{prelude::*, BoxExt, ButtonExt, WidgetExt, WidgetExtManual};

use pw_gix::gtkx::coloured::Colourable;
use pw_gix::gtkx::entry::{RGBEntryInterface, RGBHexEntryBox};
use pw_gix::gtkx::menu::WrappedMenu;
use pw_gix::wrapper::*;

use apaint_gtk_boilerplate::{Wrapper, PWO};

use crate::angles::Degrees;
use crate::attributes::ColourAttributeDisplayStack;
use crate::colour::*;
use apaint_cairo::Point;

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
    pixbuf: gdk_pixbuf::Pixbuf,
    position: Point,
}

#[derive(PWO, Wrapper)]
pub struct ColourEditor {
    vbox: gtk::Box,
    rgb_manipulator: RefCell<RGBManipulator>,
    cads: ColourAttributeDisplayStack,
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
    samples: RefCell<Vec<Sample>>,
    auto_match_btn: gtk::Button,
    auto_match_on_paste_btn: gtk::CheckButton,
    popup_menu: WrappedMenu,
    popup_menu_posn: Cell<Point>,
}

impl ColourEditor {
    pub fn new(scalar_attributes: &[ScalarAttribute], extra_buttons: &[gtk::Button]) -> Rc<Self> {
        let ced = Rc::new(Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            rgb_manipulator: RefCell::new(RGBManipulator::new()),
            cads: ColourAttributeDisplayStack::new(scalar_attributes),
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
            samples: RefCell::new(vec![]),
            auto_match_btn: gtk::Button::new_with_label("Auto Match"),
            auto_match_on_paste_btn: gtk::CheckButton::new_with_label("On Paste?"),
            popup_menu: WrappedMenu::new(&vec![]),
            popup_menu_posn: Cell::new((0.0, 0.0).into()),
        });

        let events = gdk::EventMask::BUTTON_PRESS_MASK;
        ced.drawing_area.add_events(events);
        ced.drawing_area.set_size_request(200, 200);

        ced.vbox.pack_start(&ced.cads.pwo(), false, false, 0);
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

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        for button in extra_buttons.iter() {
            hbox.pack_start(button, true, true, 0);
        }
        hbox.pack_start(&ced.auto_match_btn, true, true, 0);
        hbox.pack_start(&ced.auto_match_on_paste_btn, false, false, 0);
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
        let ced_c = Rc::clone(&ced);
        ced.vbox.connect_key_press_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::enums::key::Shift_L {
                ced_c.delta_size.set(DeltaSize::Large);
            } else if key == gdk::enums::key::Shift_R {
                ced_c.delta_size.set(DeltaSize::Small);
            };
            gtk::Inhibit(false)
        });
        let ced_c = Rc::clone(&ced);
        ced.vbox.connect_key_release_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::enums::key::Shift_L || key == gdk::enums::key::Shift_R {
                ced_c.delta_size.set(DeltaSize::Normal);
            };
            gtk::Inhibit(false)
        });
        let ced_c = Rc::clone(&ced);
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

        let ced_c = Rc::clone(&ced);
        ced.auto_match_btn
            .connect_clicked(move |_| ced_c.auto_match_samples());

        // POPUP
        let ced_c = Rc::clone(&ced);
        ced.popup_menu
            .append_item(
                "paste",
                "Paste Sample",
                "Paste image sample from the clipboard at this position",
            )
            .connect_activate(move |_| {
                let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                if let Some(pixbuf) = cbd.wait_for_image() {
                    let sample = Sample {
                        pixbuf,
                        position: ced_c.popup_menu_posn.get(),
                    };
                    ced_c.samples.borrow_mut().push(sample);
                    if ced_c.auto_match_on_paste_btn.get_active() {
                        ced_c.auto_match_samples();
                    } else {
                        ced_c.drawing_area.queue_draw();
                    };
                    ced_c.auto_match_btn.set_sensitive(true);
                } else {
                    ced_c.inform_user("No image data on clipboard.", None);
                }
            });
        let ced_c = Rc::clone(&ced);
        ced.popup_menu
            .append_item(
                "remove",
                "Remove Sample(s)",
                "Remove all image samples from the sample area",
            )
            .connect_activate(move |_| {
                ced_c.samples.borrow_mut().clear();
                ced_c.drawing_area.queue_draw();
                ced_c.auto_match_btn.set_sensitive(false);
            });
        let ced_c = Rc::clone(&ced);
        ced.drawing_area
            .connect_button_press_event(move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let position = Point::from(event.get_position());
                        let n_samples = ced_c.samples.borrow().len();
                        let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                        ced_c
                            .popup_menu
                            .set_sensitivities(cbd.wait_is_image_available(), &["paste"]);
                        ced_c
                            .popup_menu
                            .set_sensitivities(n_samples > 0, &["remove"]);
                        ced_c.popup_menu_posn.set(position);
                        ced_c.popup_menu.popup_at_event(event);
                        return Inhibit(true);
                    }
                }
                Inhibit(false)
            });

        ced
    }
}

impl ColourEditor {
    fn set_rgb(&self, rgb: RGB) {
        self.rgb_entry.set_rgb(rgb);
        self.rgb_manipulator.borrow_mut().set_rgb(rgb);
        self.cads.set_colour(Some(&rgb));
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
            let low_chroma_rgb = rgb * 0.8 + rgb.monochrome_rgb() * 0.2;
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
        cairo_context.set_source_rgb(rgb[0], rgb[1], rgb[2]);
        cairo_context.paint();
        for sample in self.samples.borrow().iter() {
            let buffer = sample
                .pixbuf
                .save_to_bufferv("png", &[])
                .expect("pixbuf to png error");
            let mut reader = std::io::Cursor::new(buffer);
            let surface = cairo::ImageSurface::create_from_png(&mut reader).unwrap();
            cairo_context.set_source_surface(&surface, sample.position.x, sample.position.y);
            cairo_context.paint();
        }
    }

    fn auto_match_samples(&self) {
        let mut red: u64 = 0;
        let mut green: u64 = 0;
        let mut blue: u64 = 0;
        let mut npixels: u32 = 0;
        for sample in self.samples.borrow().iter() {
            assert_eq!(sample.pixbuf.get_bits_per_sample(), 8);
            let nc = sample.pixbuf.get_n_channels() as usize;
            let rs = sample.pixbuf.get_rowstride() as usize;
            let width = sample.pixbuf.get_width() as usize;
            let n_rows = sample.pixbuf.get_height() as usize;
            unsafe {
                let data = sample.pixbuf.get_pixels();
                for row_num in 0..n_rows {
                    let row_start = row_num * rs;
                    let row_end = row_start + width * nc;
                    for chunk in (&data[row_start..row_end]).chunks(nc) {
                        red += chunk[0] as u64;
                        green += chunk[1] as u64;
                        blue += chunk[2] as u64;
                    }
                }
            }
            npixels += (width * n_rows) as u32;
        }
        if npixels > 0 {
            let divisor = (npixels * 255) as f64;
            let array: [f64; 3] = [
                red as f64 / divisor,
                green as f64 / divisor,
                blue as f64 / divisor,
            ];
            self.set_rgb_and_inform(array.into());
        }
    }

    pub fn reset(&self) {
        self.samples.borrow_mut().clear();
        self.set_rgb_and_inform(RGB::WHITE * 0.5);
    }

    pub fn rgb(&self) -> RGB {
        self.rgb_manipulator.borrow().rgb()
    }
}
