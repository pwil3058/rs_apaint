// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod attributes;
pub mod characteristics;
pub mod colour_mix;
pub mod drawing;
pub mod graticule;

pub use colour_math::*;
pub use float_plus::*;
pub use normalised_angles::*;

pub trait TooltipText {
    fn tooltip_text(&self) -> Option<String>;
}

impl<F: ColourComponent> TooltipText for RGB<F> {
    fn tooltip_text(&self) -> Option<String> {
        Some(format!("RGB: {}", self.pango_string()))
    }
}

pub trait ColouredItem<F: ColourComponent>: ColourInterface<F> + TooltipText {}
