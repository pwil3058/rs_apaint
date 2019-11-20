// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

pub mod attributes;
pub mod basic_paint;
pub mod characteristics;
pub mod colour_mix;
pub mod drawing;
pub mod hue_wheel;

use apaint_boilerplate::Colour;

pub use colour_math::*;
pub use float_plus::*;
pub use normalised_angles::*;

use crate::characteristics::*;

pub trait TooltipText {
    fn tooltip_text(&self) -> Option<String>;
}

impl<F: ColourComponent> TooltipText for RGB<F> {
    fn tooltip_text(&self) -> Option<String> {
        Some(format!("RGB: {}", self.pango_string()))
    }
}

pub trait Identity {
    fn id(&self) -> String;
}

impl<F: ColourComponent> Identity for RGB<F> {
    fn id(&self) -> String {
        format!("ID: {}", self.pango_string())
    }
}

pub trait BasicPaintIfce<F: ColourComponent>: ColourInterface<F> + Identity {
    fn name(&self) -> Option<&str> {
        None
    }

    fn notes(&self) -> Option<&str> {
        None
    }

    fn finish(&self) -> Finish {
        Finish::default()
    }

    fn transparency(&self) -> Transparency {
        Transparency::default()
    }

    fn fluorescence(&self) -> Fluorescence {
        Fluorescence::default()
    }

    fn permanence(&self) -> Permanence {
        Permanence::default()
    }

    fn metallicness(&self) -> Metallicness {
        Metallicness::default()
    }

    fn characteristic_abbrev(&self, characteristic_type: CharacteristicType) -> &'static str {
        match characteristic_type {
            CharacteristicType::Finish => self.finish().abbrev(),
            CharacteristicType::Transparency => self.transparency().abbrev(),
            CharacteristicType::Permanence => self.permanence().abbrev(),
            CharacteristicType::Fluorescence => self.fluorescence().abbrev(),
            CharacteristicType::Metallicness => self.metallicness().abbrev(),
        }
    }
}

impl<F: ColourComponent> TooltipText for dyn BasicPaintIfce<F> {
    fn tooltip_text(&self) -> Option<String> {
        if let Some(name) = self.name() {
            if let Some(notes) = self.notes() {
                Some(format!("{}: {}\n{}", self.id(), name, notes))
            } else {
                Some(format!("{}: {}", self.id(), name))
            }
        } else if let Some(notes) = self.notes() {
            Some(format!("{}: {}", self.id(), notes))
        } else {
            Some(format!("{}: {}", self.id(), self.rgb().pango_string()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Colour)]
struct Paint<F: ColourComponent> {
    rgb: RGB<F>,
    id: String,
}
