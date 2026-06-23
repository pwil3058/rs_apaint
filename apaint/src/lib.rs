/*
 * Copyright (c) 2026. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
 * Morbi non lorem porttitor neque feugiat blandit. Ut vitae ipsum eget quam lacinia accumsan.
 * Etiam sed turpis ac ipsum condimentum fringilla. Maecenas magna.
 * Proin dapibus sapien vel ante. Aliquam erat volutpat. Pellentesque sagittis ligula eget metus.
 * Vestibulum commodo. Ut rhoncus gravida arcu.
 */
#[macro_use]
extern crate serde_derive;

use std::{error, fmt, io, result};

use colour_math::{ColourAttributes, ColourBasics};

pub mod legacy;
pub mod mixtures;
pub mod properties;
pub mod series;
pub mod watercolour;

use crate::properties::*;

pub trait TooltipText {
    fn tooltip_text(&self) -> String;
}

pub trait LabelText {
    fn label_text(&self) -> String;
}

pub trait BasicPaintIfce: ColourBasics + ColourAttributes {
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

    fn opacity(&self) -> Opacity {
        Opacity::default()
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

    fn light_fastness(&self) -> LightFastness {
        LightFastness::default()
    }

    fn staining(&self) -> Staining {
        Staining::default()
    }

    fn granulation(&self) -> Granulation {
        Granulation::default()
    }

    fn property(&self, property_type: PropertyType) -> Property {
        match property_type {
            PropertyType::Finish => Property::Finish(self.finish()),
            PropertyType::Transparency => Property::Transparency(self.transparency()),
            PropertyType::Permanence => Property::Permanence(self.permanence()),
            PropertyType::Fluorescence => Property::Fluorescence(self.fluorescence()),
            PropertyType::Metallicness => Property::Metallicness(self.metallicness()),
            PropertyType::Staining => Property::Staining(self.staining()),
            PropertyType::LightFastness => Property::LightFastness(self.light_fastness()),
            PropertyType::Granulation => Property::Granulation(self.granulation()),
            PropertyType::Opacity => Property::Granulation(self.granulation()),
        }
    }
}

pub trait WatercolourIfce: ColourBasics + ColourAttributes {
    fn id(&self) -> &str;

    fn name(&self) -> Option<&str> {
        None
    }

    fn notes(&self) -> Option<&str> {
        None
    }

    fn transparency(&self) -> Transparency {
        Transparency::default()
    }

    fn fluorescence(&self) -> Fluorescence {
        Fluorescence::default()
    }

    fn light_fastness(&self) -> LightFastness {
        LightFastness::default()
    }

    fn staining(&self) -> Staining {
        Staining::default()
    }

    fn granulation(&self) -> Granulation {
        Granulation::default()
    }

    fn property(&self, property_type: WatercolourPropertyType) -> Property {
        match property_type {
            WatercolourPropertyType::Transparency => Property::Transparency(self.transparency()),
            WatercolourPropertyType::Fluorescence => Property::Fluorescence(self.fluorescence()),
            WatercolourPropertyType::Staining => Property::Staining(self.staining()),
            WatercolourPropertyType::LightFastness => {
                Property::LightFastness(self.light_fastness())
            }
            WatercolourPropertyType::Granulation => Property::Granulation(self.granulation()),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    SerdeJsonError(serde_json::Error),
    NotFound(String),
    UnknownSeries(series::SeriesId),
    UnknownSeriesPaint(series::SeriesId, String),
    NotAValidLegacySpec,
    NotImplemented,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(err) => write!(f, "IOError: {err}"),
            Error::SerdeJsonError(err) => write!(f, "Serde Json Error: {err}"),
            Error::NotFound(string) => write!(f, "{string}: Not found."),
            Error::UnknownSeries(series_id) => write!(f, "{series_id}: unknown paint series"),
            Error::UnknownSeriesPaint(series_id, id) => {
                write!(f, "{id}:({series_id}): unknown paint")
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
