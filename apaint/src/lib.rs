// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

pub mod attributes;
pub mod characteristics;
pub mod colour_mix;
pub mod drawing;
pub mod hue_wheel;

use apaint_boilerplate::Colour;

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

#[derive(Debug, Serialize, Deserialize, Colour)]
struct Paint<F: ColourComponent> {
    rgb: RGB<F>,
    id: String,
}
