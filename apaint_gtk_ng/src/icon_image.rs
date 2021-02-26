// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use pw_gix::{gdk_pixbuf, gtk};

use apaint_ng::xpm::*;

// NEW COLLECTION
pub fn colln_new_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(COLLN_NEW_XPM)
}

pub fn colln_new_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        colln_new_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// LOAD
pub fn colln_load_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(COLLN_LOAD_XPM)
}

pub fn colln_load_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        colln_load_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// SAVE
pub fn colln_save_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(COLLN_SAVE_XPM)
}

pub fn colln_save_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        colln_save_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// SAVE AS
pub fn colln_save_as_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(COLLN_SAVE_AS_XPM)
}

pub fn colln_save_as_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        colln_save_as_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// UP TO DATE
pub fn up_to_date_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(UP_TO_DATE_XPM)
}

pub fn up_to_date_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        up_to_date_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// NEEDS SAVE AND READY
pub fn needs_save_ready_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(NEEDS_SAVE_READY_XPM)
}

pub fn needs_save_ready_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        needs_save_ready_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// NEEDS SAVE BUT NOT READY
pub fn needs_save_not_ready_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(NEEDS_SAVE_NOT_READY_XPM)
}

pub fn needs_save_not_ready_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        needs_save_not_ready_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// SERIES PAINT IMAGE
pub fn series_paint_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(SERIES_PAINT_XPM)
}

pub fn series_paint_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        series_paint_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// SERIES PAINT LOAD IMAGE
pub fn series_paint_load_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(SERIES_PAINT_LOAD_XPM)
}

pub fn series_paint_load_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        series_paint_load_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// PAINT STANDARD IMAGE
pub fn paint_standard_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(PAINT_STANDARD_XPM)
}

pub fn paint_standard_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        paint_standard_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

// PAINT STANDARD LOAD IMAGE
pub fn paint_standard_load_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::from_xpm_data(PAINT_STANDARD_LOAD_XPM)
}

pub fn paint_standard_load_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        paint_standard_load_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}
