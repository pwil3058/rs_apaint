// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk::WidgetExt;

use apaint_gtk_boilerplate::{Wrapper, PWO};
use pw_gix::wrapper::*;

use crate::drawing::Drawer;
use apaint::attributes::ColourAttributeDisplayIfce;

#[derive(PWO, Wrapper)]
pub struct ColourAttributeDisplay<A: ColourAttributeDisplayIfce<f64>> {
    drawing_area: gtk::DrawingArea,
    drawer: Drawer,
    attribute: A,
}
