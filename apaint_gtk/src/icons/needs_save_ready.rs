// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub static XPM: &[&str] = &[
    "64 64 2 1",
    " 	c None",
    "1	c #FFAF00",
    "                          111111111111                          ",
    "                      11111111111111111111                      ",
    "                    111111111111111111111111                    ",
    "                  1111111111111111111111111111                  ",
    "                11111111111111111111111111111111                ",
    "              111111111111111111111111111111111111              ",
    "             11111111111111111111111111111111111111             ",
    "            1111111111111111111111111111111111111111            ",
    "          11111111111111111111111111111111111111111111          ",
    "         1111111111111111111111111111111111111111111111         ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "       11111111111111111111111111111111111111111111111111       ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "       11111111111111111111111111111111111111111111111111       ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "         1111111111111111111111111111111111111111111111         ",
    "          11111111111111111111111111111111111111111111          ",
    "            1111111111111111111111111111111111111111            ",
    "             11111111111111111111111111111111111111             ",
    "              111111111111111111111111111111111111              ",
    "                11111111111111111111111111111111                ",
    "                  1111111111111111111111111111                  ",
    "                    111111111111111111111111                    ",
    "                      11111111111111111111                      ",
    "                          111111111111                          ",
];

use pw_gix::{gdk_pixbuf, gtk};

#[allow(dead_code)]
pub fn pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(XPM)
}

#[allow(dead_code)]
pub fn sized_pixbuf(size: i32) -> Option<gdk_pixbuf::Pixbuf> {
    pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
}

#[allow(dead_code)]
pub fn sized_pixbuf_or(size: i32) -> gdk_pixbuf::Pixbuf {
    if let Some(pixbuf) = sized_pixbuf(size) {
        pixbuf
    } else {
        pixbuf()
    }
}

#[allow(dead_code)]
pub fn image() -> gtk::Image {
    gtk::Image::from_pixbuf(Some(&pixbuf()))
}

#[allow(dead_code)]
pub fn sized_image(size: i32) -> Option<gtk::Image> {
    if let Some(pixbuf) = pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear) {
        Some(gtk::Image::from_pixbuf(Some(&pixbuf)))
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn sized_image_or(size: i32) -> gtk::Image {
    if let Some(image) = sized_image(size) {
        image
    } else {
        image()
    }
}
