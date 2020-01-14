// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk::prelude::*;

use pw_gix::{gtkx::window::RememberGeometry, recollections};

mod config;

fn main() {
    if let Err(err) = gtk::init() {
        panic!("GTK failed to initialize! {}.", err);
    };
    recollections::init(&config::recollection_file_path());
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_geometry_from_recollections("main_window", (600, 400));
    if let Some(icon) = icon::mcmmtkrs_pixbuf(64) {
        win.set_icon(Some(&icon));
    }
    win.set_title("Modellers Colour Mixing/Matching Tool Kit");
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}

mod icon {
    use gdk_pixbuf;
    use gtk;

    // XPM
    static MCMMTKRS_XPM: &[&str] = &[
        "8 8 3 1",
        "R c #FF0000",
        "Y c #FFFF00",
        "_ c #000000",
        "RRRR____",
        "RRYYRRYY",
        "RRRR____",
        "RRRR____",
        "RR______",
        "RRYYRR__",
        "YYRRYY__",
        "YYYYYY__",
    ];

    pub fn mcmmtkrs_pixbuf(size: i32) -> Option<gdk_pixbuf::Pixbuf> {
        gdk_pixbuf::Pixbuf::new_from_xpm_data(MCMMTKRS_XPM).scale_simple(
            size,
            size,
            gdk_pixbuf::InterpType::Tiles,
        )
    }

    pub fn _mcmmtkrs_image(size: i32) -> Option<gtk::Image> {
        if let Some(pixbuf) = mcmmtkrs_pixbuf(size) {
            Some(gtk::Image::new_from_pixbuf(Some(&pixbuf)))
        } else {
            None
        }
    }
}
