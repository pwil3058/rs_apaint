// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

//! Types to describe paint characteristics that cannot be derived from their colour.

use std::{fmt, str::FromStr};

use apaint_boilerplate::Characteristic;

pub trait CharacteristicIfce:
    FromStr<Err = String> + PartialEq + PartialOrd + Default + fmt::Debug
{
    const NAME: &'static str;
    const PROMPT: &'static str;
    const LIST_HEADER_NAME: &'static str;

    fn str_values() -> Vec<&'static str>;

    fn abbrev(&self) -> &'static str;
    fn full(&self) -> &'static str;
}

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Characteristic,
)]
pub enum Finish {
    Gloss,
    SemiGloss,
    SemiFlat,
    Flat,
}

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Characteristic,
)]
pub enum Transparency {
    Opaque,
    SemiOpaque,
    SemiTransparent,
    Transparent,
    Clear,
}

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Characteristic,
)]
pub enum Permanence {
    ExtremelyPermanent,
    #[default]
    Permanent,
    ModeratelyDurable,
    Fugitive,
}

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Characteristic,
)]
pub enum Fluorescence {
    Fluorescent,
    SemiFluorescent,
    SemiNonFluorescent,
    #[default]
    NonFluorescent,
}

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Characteristic,
)]
pub enum Metallicness {
    Metal,
    Metallic,
    SemiMetallic,
    #[default]
    NonMetallic,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Characteristic {
    Finish(Finish),
    Transparency(Transparency),
    Permanence(Permanence),
    Fluorescence(Fluorescence),
    Metallicness(Metallicness),
}

impl Characteristic {
    pub fn name(self) -> &'static str {
        match self {
            Characteristic::Finish(_) => Finish::NAME,
            Characteristic::Transparency(_) => Transparency::NAME,
            Characteristic::Permanence(_) => Permanence::NAME,
            Characteristic::Fluorescence(_) => Fluorescence::NAME,
            Characteristic::Metallicness(_) => Metallicness::NAME,
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            Characteristic::Finish(_) => Finish::PROMPT,
            Characteristic::Transparency(_) => Transparency::PROMPT,
            Characteristic::Permanence(_) => Permanence::PROMPT,
            Characteristic::Fluorescence(_) => Fluorescence::PROMPT,
            Characteristic::Metallicness(_) => Metallicness::PROMPT,
        }
    }

    pub fn list_header_name(self) -> &'static str {
        match self {
            Characteristic::Finish(_) => Finish::LIST_HEADER_NAME,
            Characteristic::Transparency(_) => Transparency::LIST_HEADER_NAME,
            Characteristic::Permanence(_) => Permanence::LIST_HEADER_NAME,
            Characteristic::Fluorescence(_) => Fluorescence::LIST_HEADER_NAME,
            Characteristic::Metallicness(_) => Metallicness::LIST_HEADER_NAME,
        }
    }

    pub fn str_values(self) -> Vec<&'static str> {
        match self {
            Characteristic::Finish(_) => Finish::str_values(),
            Characteristic::Transparency(_) => Transparency::str_values(),
            Characteristic::Permanence(_) => Permanence::str_values(),
            Characteristic::Fluorescence(_) => Fluorescence::str_values(),
            Characteristic::Metallicness(_) => Metallicness::str_values(),
        }
    }

    pub fn abbrev(self) -> &'static str {
        match self {
            Characteristic::Finish(value) => value.abbrev(),
            Characteristic::Transparency(value) => value.abbrev(),
            Characteristic::Permanence(value) => value.abbrev(),
            Characteristic::Fluorescence(value) => value.abbrev(),
            Characteristic::Metallicness(value) => value.abbrev(),
        }
    }
    pub fn full(self) -> &'static str {
        match self {
            Characteristic::Finish(value) => value.full(),
            Characteristic::Transparency(value) => value.full(),
            Characteristic::Permanence(value) => value.full(),
            Characteristic::Fluorescence(value) => value.full(),
            Characteristic::Metallicness(value) => value.full(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum CharacteristicType {
    Finish,
    Transparency,
    Permanence,
    Fluorescence,
    Metallicness,
}

impl CharacteristicType {
    pub fn name(self) -> &'static str {
        match self {
            CharacteristicType::Finish => Finish::NAME,
            CharacteristicType::Transparency => Transparency::NAME,
            CharacteristicType::Permanence => Permanence::NAME,
            CharacteristicType::Fluorescence => Fluorescence::NAME,
            CharacteristicType::Metallicness => Metallicness::NAME,
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            CharacteristicType::Finish => Finish::PROMPT,
            CharacteristicType::Transparency => Transparency::PROMPT,
            CharacteristicType::Permanence => Permanence::PROMPT,
            CharacteristicType::Fluorescence => Fluorescence::PROMPT,
            CharacteristicType::Metallicness => Metallicness::PROMPT,
        }
    }

    pub fn list_header_name(self) -> &'static str {
        match self {
            CharacteristicType::Finish => Finish::LIST_HEADER_NAME,
            CharacteristicType::Transparency => Transparency::LIST_HEADER_NAME,
            CharacteristicType::Permanence => Permanence::LIST_HEADER_NAME,
            CharacteristicType::Fluorescence => Fluorescence::LIST_HEADER_NAME,
            CharacteristicType::Metallicness => Metallicness::LIST_HEADER_NAME,
        }
    }

    pub fn str_values(self) -> Vec<&'static str> {
        match self {
            CharacteristicType::Finish => Finish::str_values(),
            CharacteristicType::Transparency => Transparency::str_values(),
            CharacteristicType::Permanence => Permanence::str_values(),
            CharacteristicType::Fluorescence => Fluorescence::str_values(),
            CharacteristicType::Metallicness => Metallicness::str_values(),
        }
    }
}

impl std::fmt::Display for CharacteristicType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CharacteristicType::Finish => write!(f, "Finish"),
            CharacteristicType::Transparency => write!(f, "Transparency"),
            CharacteristicType::Permanence => write!(f, "Permanence"),
            CharacteristicType::Fluorescence => write!(f, "Fluorescence"),
            CharacteristicType::Metallicness => write!(f, " Metallicness"),
        }
    }
}

#[derive(Debug, Default)]
pub struct CharacteristicMixer<C: CharacteristicIfce> {
    sum: f64,
    total_parts: u64,
    phantom: std::marker::PhantomData<C>,
}

impl<C: CharacteristicIfce + From<f64> + Into<f64>> CharacteristicMixer<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.sum = 0.0;
        self.total_parts = 0;
    }

    pub fn characteristic(&self) -> Option<C> {
        if self.total_parts > 0 {
            let mixture = self.sum / self.total_parts as f64;
            Some(C::from(mixture))
        } else {
            None
        }
    }

    pub fn add(&mut self, characteristic: C, parts: u64) {
        self.total_parts += parts;
        self.sum += characteristic.into() * parts as f64;
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
        assert_eq!(Transparency::default(), Transparency::Opaque);
        assert_eq!(Permanence::default(), Permanence::Permanent);
        assert_eq!(Fluorescence::default(), Fluorescence::NonFluorescent);
        assert_eq!(Metallicness::default(), Metallicness::NonMetallic);
    }

    #[test]
    fn mixture() {
        let mut mixer = CharacteristicMixer::<Finish>::new();
        assert_eq!(mixer.characteristic(), None);
        mixer.add(Finish::Gloss, 1);
        mixer.add(Finish::Flat, 10);
        assert_eq!(mixer.characteristic(), Some(Finish::Flat));
        mixer.add(Finish::Gloss, 6);
        assert_eq!(mixer.characteristic(), Some(Finish::SemiFlat));
    }
}
