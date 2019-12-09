// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

use std::{error, fmt, io};

use serde_json;

pub mod attributes;
pub mod basic_paint;
pub mod characteristics;
pub mod colour_mix;
pub mod drawing;
pub mod hue_wheel;
pub mod series;
pub mod spec;
pub mod xpm;

pub use colour_math::*;
pub use float_plus::*;
pub use normalised_angles::*;

use crate::characteristics::*;

pub trait TooltipText {
    fn tooltip_text(&self) -> String;
}

pub trait LabelText {
    fn label_text(&self) -> String;
}

pub trait BasicPaintIfce<F: ColourComponent>: ColourInterface<F> {
    fn id(&self) -> &str;

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

pub trait FromSpec<F: ColourComponent> {
    fn from_spec(spec: &spec::BasicPaintSpec<F>) -> Self;
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    SerdeJsonError(serde_json::Error),
    NotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(err) => write!(f, "IOError: {}", err),
            Error::SerdeJsonError(err) => write!(f, "Serde Json Error: {}", err),
            Error::NotFound(string) => write!(f, "{}: Not found.", string),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IOError(err) => Some(err),
            Error::SerdeJsonError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJsonError(err)
    }
}
