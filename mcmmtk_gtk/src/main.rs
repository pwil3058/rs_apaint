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
    win.set_geometry_from_recollections("main_window", (200, 200));
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

    pub fn mcmmtkrs_pixbuf() -> gdk_pixbuf::Pixbuf {
        gdk_pixbuf::Pixbuf::new_from_xpm_data(MCMMTKRS_XPM)
    }

    pub fn mcmmtkrs_image(size: i32) -> gtk::Image {
        if let Some(pixbuf) =
            mcmmtkrs_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
        {
            gtk::Image::new_from_pixbuf(Some(&pixbuf))
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }
}
