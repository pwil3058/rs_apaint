/*
 * Copyright (c) 2026. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
 * Morbi non lorem porttitor neque feugiat blandit. Ut vitae ipsum eget quam lacinia accumsan.
 * Etiam sed turpis ac ipsum condimentum fringilla. Maecenas magna.
 * Proin dapibus sapien vel ante. Aliquam erat volutpat. Pellentesque sagittis ligula eget metus.
 * Vestibulum commodo. Ut rhoncus gravida arcu.
 */

//! Types to describe paint properties that cannot be derived from their colour.

use std::{fmt, str::FromStr};

use apaint_boilerplate::Property;
use std::marker::PhantomData;

pub trait PropertyIfce:
    FromStr<Err = String> + PartialEq + PartialOrd + Default + fmt::Debug
{
    const NAME: &'static str;
    const PROMPT: &'static str;
    const LIST_HEADER_NAME: &'static str;

    fn str_values() -> Vec<&'static str>;

    fn abbrev(&self) -> &'static str;
    fn full(&self) -> &'static str;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct FuzzyProperty<C: PropertyIfce>(f64, std::marker::PhantomData<C>);

impl<C: PropertyIfce + Into<f64>> std::str::FromStr for FuzzyProperty<C> {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, String> {
        let property = C::from_str(string)?;
        Ok(Self(property.into(), std::marker::PhantomData))
    }
}

impl<C: PropertyIfce + From<f64>> FuzzyProperty<C> {
    pub fn property(self) -> C {
        C::from(self.0)
    }
}

impl<C: PropertyIfce + Into<f64> + From<f64>> PropertyIfce for FuzzyProperty<C> {
    const NAME: &'static str = C::NAME;
    const PROMPT: &'static str = C::PROMPT;
    const LIST_HEADER_NAME: &'static str = C::LIST_HEADER_NAME;

    fn str_values() -> Vec<&'static str> {
        C::str_values()
    }

    fn abbrev(&self) -> &'static str {
        C::from(self.0).abbrev()
    }

    fn full(&self) -> &'static str {
        C::from(self.0).full()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Finish {
    Gloss,
    SemiGloss,
    SemiFlat,
    Flat,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Opacity {
    Opaque,
    SemiOpaque,
    SemiTransparent,
    Transparent,
    Clear,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Transparency {
    Opaque,
    SemiOpaque,
    SemiTransparent,
    #[default]
    Transparent,
    Clear,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Permanence {
    ExtremelyPermanent,
    #[default]
    Permanent,
    ModeratelyDurable,
    Fugitive,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Fluorescence {
    Fluorescent,
    SemiFluorescent,
    SemiNonFluorescent,
    #[default]
    NonFluorescent,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Metallicness {
    Metal,
    Metallic,
    SemiMetallic,
    #[default]
    NonMetallic,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum LightFastness {
    Excellent,
    #[default]
    VeryGood,
    Fair,
    Fugitive,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Staining {
    HighStaining,
    #[default]
    ModerateStaining,
    LowStaining,
    NonStaining,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Property)]
pub enum Granulation {
    Granulating,
    SomeGranulation,
    #[default]
    NonGranulating,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Property {
    Finish(Finish),
    Transparency(Transparency),
    Permanence(Permanence),
    Fluorescence(Fluorescence),
    Metallicness(Metallicness),
    LightFastness(LightFastness),
    Opacity(Opacity),
    Staining(Staining),
    Granulation(Granulation),
}

impl Property {
    pub fn name(self) -> &'static str {
        use PropertyIfce;
        match self {
            Self::Finish(_) => Finish::NAME,
            Self::Transparency(_) => Transparency::NAME,
            Self::Permanence(_) => Permanence::NAME,
            Self::Fluorescence(_) => Fluorescence::NAME,
            Self::Metallicness(_) => Metallicness::NAME,
            Self::LightFastness(_) => LightFastness::NAME,
            Self::Opacity(_) => Opacity::NAME,
            Self::Staining(_) => Staining::NAME,
            Self::Granulation(_) => Granulation::NAME,
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            Self::Finish(_) => Finish::PROMPT,
            Self::Transparency(_) => Transparency::PROMPT,
            Self::Permanence(_) => Permanence::PROMPT,
            Self::Fluorescence(_) => Fluorescence::PROMPT,
            Self::Metallicness(_) => Metallicness::PROMPT,
            Self::LightFastness(_) => LightFastness::PROMPT,
            Self::Opacity(_) => Opacity::PROMPT,
            Self::Staining(_) => Staining::PROMPT,
            Self::Granulation(_) => Granulation::PROMPT,
        }
    }

    pub fn list_header_name(self) -> &'static str {
        match self {
            Self::Finish(_) => Finish::LIST_HEADER_NAME,
            Self::Transparency(_) => Transparency::LIST_HEADER_NAME,
            Self::Permanence(_) => Permanence::LIST_HEADER_NAME,
            Self::Fluorescence(_) => Fluorescence::LIST_HEADER_NAME,
            Self::Metallicness(_) => Metallicness::LIST_HEADER_NAME,
            Self::LightFastness(_) => LightFastness::LIST_HEADER_NAME,
            Self::Opacity(_) => Opacity::LIST_HEADER_NAME,
            Self::Staining(_) => Staining::LIST_HEADER_NAME,
            Self::Granulation(_) => Granulation::LIST_HEADER_NAME,
        }
    }

    pub fn str_values(self) -> Vec<&'static str> {
        match self {
            Self::Finish(_) => Finish::str_values(),
            Self::Transparency(_) => Transparency::str_values(),
            Self::Permanence(_) => Permanence::str_values(),
            Self::Fluorescence(_) => Fluorescence::str_values(),
            Self::Metallicness(_) => Metallicness::str_values(),
            Self::LightFastness(_) => LightFastness::str_values(),
            Self::Opacity(_) => Opacity::str_values(),
            Self::Staining(_) => Staining::str_values(),
            Self::Granulation(_) => Granulation::str_values(),
        }
    }

    pub fn abbrev(self) -> &'static str {
        match self {
            Self::Finish(value) => value.abbrev(),
            Self::Transparency(value) => value.abbrev(),
            Self::Permanence(value) => value.abbrev(),
            Self::Fluorescence(value) => value.abbrev(),
            Self::Metallicness(value) => value.abbrev(),
            Self::LightFastness(value) => value.abbrev(),
            Self::Opacity(value) => value.abbrev(),
            Self::Staining(value) => value.abbrev(),
            Self::Granulation(value) => value.abbrev(),
        }
    }
    pub fn full(self) -> &'static str {
        match self {
            Self::Finish(value) => value.full(),
            Self::Transparency(value) => value.full(),
            Self::Permanence(value) => value.full(),
            Self::Fluorescence(value) => value.full(),
            Self::Metallicness(value) => value.full(),
            Self::LightFastness(value) => value.full(),
            Self::Opacity(value) => value.full(),
            Self::Staining(value) => value.full(),
            Self::Granulation(value) => value.full(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum PropertyType {
    Finish,
    Transparency,
    Permanence,
    Fluorescence,
    Metallicness,
    LightFastness,
    Opacity,
    Staining,
    Granulation,
}

impl PropertyType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Finish => Finish::NAME,
            Self::Transparency => Transparency::NAME,
            Self::Permanence => Permanence::NAME,
            Self::Fluorescence => Fluorescence::NAME,
            Self::Metallicness => Metallicness::NAME,
            Self::LightFastness => LightFastness::NAME,
            Self::Opacity => Opacity::NAME,
            Self::Staining => Staining::NAME,
            Self::Granulation => Granulation::NAME,
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            Self::Finish => Finish::PROMPT,
            Self::Transparency => Transparency::PROMPT,
            Self::Permanence => Permanence::PROMPT,
            Self::Fluorescence => Fluorescence::PROMPT,
            Self::Metallicness => Metallicness::PROMPT,
            Self::LightFastness => LightFastness::PROMPT,
            Self::Opacity => Opacity::PROMPT,
            Self::Staining => Staining::PROMPT,
            Self::Granulation => Granulation::PROMPT,
        }
    }

    pub fn list_header_name(self) -> &'static str {
        match self {
            Self::Finish => Finish::LIST_HEADER_NAME,
            Self::Transparency => Transparency::LIST_HEADER_NAME,
            Self::Permanence => Permanence::LIST_HEADER_NAME,
            Self::Fluorescence => Fluorescence::LIST_HEADER_NAME,
            Self::Metallicness => Metallicness::LIST_HEADER_NAME,
            Self::LightFastness => LightFastness::LIST_HEADER_NAME,
            Self::Opacity => Opacity::LIST_HEADER_NAME,
            Self::Staining => Staining::LIST_HEADER_NAME,
            Self::Granulation => Granulation::LIST_HEADER_NAME,
        }
    }

    pub fn str_values(self) -> Vec<&'static str> {
        match self {
            Self::Finish => Finish::str_values(),
            Self::Transparency => Transparency::str_values(),
            Self::Permanence => Permanence::str_values(),
            Self::Fluorescence => Fluorescence::str_values(),
            Self::Metallicness => Metallicness::str_values(),
            Self::LightFastness => LightFastness::str_values(),
            Self::Opacity => Opacity::str_values(),
            Self::Staining => Staining::str_values(),
            Self::Granulation => Granulation::str_values(),
        }
    }
}
impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Finish => write!(f, "Finish"),
            Self::Transparency => write!(f, "Transparency"),
            Self::Permanence => write!(f, "Permanence"),
            Self::Fluorescence => write!(f, "Fluorescence"),
            Self::Metallicness => write!(f, " Metallicness"),
            Self::Granulation => write!(f, " Granulation"),
            Self::Opacity => write!(f, " Opacity"),
            Self::Staining => write!(f, " Staining"),
            Self::LightFastness => write!(f, " LightFastness"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum WatercolourProperty {
    LightFastness(LightFastness),
    Transparency(Transparency),
    Staining(Staining),
    Granulation(Granulation),
    Fluorescence(Fluorescence),
}

impl WatercolourProperty {
    pub fn name(self) -> &'static str {
        match self {
            WatercolourProperty::LightFastness(_) => LightFastness::NAME,
            WatercolourProperty::Transparency(_) => Transparency::NAME,
            WatercolourProperty::Staining(_) => Staining::NAME,
            WatercolourProperty::Granulation(_) => Granulation::NAME,
            WatercolourProperty::Fluorescence(_) => Fluorescence::NAME,
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            WatercolourProperty::LightFastness(_) => LightFastness::PROMPT,
            WatercolourProperty::Transparency(_) => Transparency::PROMPT,
            WatercolourProperty::Staining(_) => Staining::PROMPT,
            WatercolourProperty::Granulation(_) => Granulation::PROMPT,
            WatercolourProperty::Fluorescence(_) => Fluorescence::PROMPT,
        }
    }

    pub fn list_header_name(self) -> &'static str {
        match self {
            WatercolourProperty::LightFastness(_) => LightFastness::LIST_HEADER_NAME,
            WatercolourProperty::Transparency(_) => Transparency::LIST_HEADER_NAME,
            WatercolourProperty::Staining(_) => Staining::LIST_HEADER_NAME,
            WatercolourProperty::Granulation(_) => Granulation::LIST_HEADER_NAME,
            WatercolourProperty::Fluorescence(_) => Fluorescence::LIST_HEADER_NAME,
        }
    }

    pub fn str_values(self) -> Vec<&'static str> {
        match self {
            WatercolourProperty::LightFastness(_) => LightFastness::str_values(),
            WatercolourProperty::Transparency(_) => Transparency::str_values(),
            WatercolourProperty::Staining(_) => Staining::str_values(),
            WatercolourProperty::Granulation(_) => Granulation::str_values(),
            WatercolourProperty::Fluorescence(_) => Fluorescence::str_values(),
        }
    }

    pub fn abbrev(self) -> &'static str {
        match self {
            WatercolourProperty::LightFastness(value) => value.abbrev(),
            WatercolourProperty::Transparency(value) => value.abbrev(),
            WatercolourProperty::Staining(value) => value.abbrev(),
            WatercolourProperty::Granulation(value) => value.abbrev(),
            WatercolourProperty::Fluorescence(value) => value.abbrev(),
        }
    }

    pub fn full(self) -> &'static str {
        match self {
            WatercolourProperty::LightFastness(value) => value.full(),
            WatercolourProperty::Transparency(value) => value.full(),
            WatercolourProperty::Staining(value) => value.full(),
            WatercolourProperty::Granulation(value) => value.full(),
            WatercolourProperty::Fluorescence(value) => value.full(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum WatercolourPropertyType {
    LightFastness,
    Transparency,
    Staining,
    Granulation,
    Fluorescence,
}

#[derive(Debug, Default)]
pub struct PropertyMixer<C: PropertyIfce> {
    sum: f64,
    total_parts: u64,
    phantom: std::marker::PhantomData<C>,
}

impl<C: PropertyIfce + From<f64> + Into<f64>> PropertyMixer<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.sum = 0.0;
        self.total_parts = 0;
    }

    pub fn property(&self) -> Option<C> {
        if self.total_parts > 0 {
            let mixture = self.sum / self.total_parts as f64;
            Some(C::from(mixture))
        } else {
            None
        }
    }

    pub fn property_value(&self) -> Option<FuzzyProperty<C>> {
        if self.total_parts > 0 {
            Some(FuzzyProperty(
                self.sum / self.total_parts as f64,
                PhantomData,
            ))
        } else {
            None
        }
    }

    pub fn add(&mut self, property: C, parts: u64) {
        self.total_parts += parts;
        self.sum += property.into() * parts as f64;
    }

    pub fn add_value(&mut self, characteristic_value: FuzzyProperty<C>, parts: u64) {
        self.total_parts += parts;
        self.sum += characteristic_value.0 * parts as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_finish() {
        assert_eq!(Finish::NAME, "Finish");
        assert_eq!(Finish::PROMPT, "Finish:");
        assert_eq!(Finish::Gloss.abbrev(), "G");
        assert_eq!(Finish::SemiGloss.abbrev(), "SG");
        assert_eq!(Finish::SemiFlat.abbrev(), "SF");
        assert_eq!(Finish::Flat.abbrev(), "F");
        assert_eq!(Finish::Gloss.full(), "gloss");
        assert_eq!(Finish::SemiGloss.full(), "semi-gloss");
        assert_eq!(Finish::SemiFlat.full(), "semi-flat");
        assert_eq!(Finish::Flat.full(), "flat");
        for a in ["G", "SG", "SF", "F"].iter() {
            assert_eq!(Finish::from_str(a).unwrap().abbrev(), *a);
        }
        for a in ["gloss", "semi-gloss", "semi-flat", "flat"].iter() {
            assert_eq!(Finish::from_str(a).unwrap().full(), *a);
        }
    }

    #[test]
    fn defaults() {
        assert_eq!(Finish::default(), Finish::Gloss);
        assert_eq!(Transparency::default(), Transparency::Transparent);
        assert_eq!(Permanence::default(), Permanence::Permanent);
        assert_eq!(Fluorescence::default(), Fluorescence::NonFluorescent);
        assert_eq!(Metallicness::default(), Metallicness::NonMetallic);
    }

    #[test]
    fn mixture() {
        let mut mixer = PropertyMixer::<Finish>::new();
        assert_eq!(mixer.property(), None);
        mixer.add(Finish::Gloss, 1);
        mixer.add(Finish::Flat, 10);
        assert_eq!(mixer.property(), Some(Finish::Flat));
        mixer.add(Finish::Gloss, 6);
        assert_eq!(mixer.property(), Some(Finish::SemiFlat));
    }
}
