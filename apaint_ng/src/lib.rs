// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[macro_use]
extern crate serde_derive;

use std::{error, fmt, io, result};

use serde_json;

pub mod characteristics;
pub mod colour_mix;
pub mod legacy;
pub mod mixtures;
pub mod series;
pub mod xpm;

pub use colour_math::*;
pub use normalised_angles::*;
pub use num_traits_plus::float_plus::*;

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

    fn characteristic(&self, characteristic_type: CharacteristicType) -> Characteristic {
        match characteristic_type {
            CharacteristicType::Finish => Characteristic::Finish(self.finish()),
            CharacteristicType::Transparency => Characteristic::Transparency(self.transparency()),
            CharacteristicType::Permanence => Characteristic::Permanence(self.permanence()),
            CharacteristicType::Fluorescence => Characteristic::Fluorescence(self.fluorescence()),
            CharacteristicType::Metallicness => Characteristic::Metallicness(self.metallicness()),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    SerdeJsonError(serde_json::Error),
    NotFound(String),
    UnknownSeries(crate::series::SeriesId),
    UnknownSeriesPaint(crate::series::SeriesId, String),
    NotAValidLegacySpec,
    NotImplemented,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(err) => write!(f, "IOError: {}", err),
            Error::SerdeJsonError(err) => write!(f, "Serde Json Error: {}", err),
            Error::NotFound(string) => write!(f, "{}: Not found.", string),
            Error::UnknownSeries(series_id) => write!(f, "{}: unknown paint series", series_id),
            Error::UnknownSeriesPaint(series_id, id) => {
                write!(f, "{}:({}): unknown paint", id, series_id)
            }
            Error::NotAValidLegacySpec => write!(f, "Not a valid specification."),
            Error::NotImplemented => write!(f, "Feature not yet implemented."),
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

pub type Result<T> = result::Result<T, Error>;
