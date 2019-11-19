// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

//! Types to describe paint characteristics that cannot be derived from their colour.

use std::str::FromStr;

use apaint_boilerplate::Characteristic;

pub trait CharacteristicIfce: FromStr + PartialEq + PartialOrd + Default {
    const NAME: &'static str;
    const PROMPT: &'static str;

    fn str_values() -> Vec<&'static str>;

    fn abbrev(&self) -> &'static str;
    fn full(&self) -> &'static str;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Characteristic)]
pub enum Finish {
    Gloss,
    SemiGloss,
    SemiFlat,
    Flat,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Characteristic)]
pub enum Transparency {
    Opaque,
    SemiOpaque,
    SemiTransparent,
    Transparent,
    Clear,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Characteristic)]
pub enum Permanence {
    ExtremelyPermanent,
    #[default]
    Permanent,
    ModeratelyDurable,
    Fugitive,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Characteristic)]
pub enum Fluorescence {
    Fluorescent,
    SemiFluorescent,
    SemiNonFluorescent,
    #[default]
    NonFluorescent,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Characteristic)]
pub enum Metallicness {
    Metal,
    Metallic,
    SemiMetallic,
    #[default]
    NonMetallic,
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
    pub fn name(&self) -> &'static str {
        match *self {
            CharacteristicType::Finish => Finish::NAME,
            CharacteristicType::Transparency => Transparency::NAME,
            CharacteristicType::Permanence => Permanence::NAME,
            CharacteristicType::Fluorescence => Fluorescence::NAME,
            CharacteristicType::Metallicness => Metallicness::NAME,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match *self {
            CharacteristicType::Finish => Finish::PROMPT,
            CharacteristicType::Transparency => Transparency::PROMPT,
            CharacteristicType::Permanence => Permanence::PROMPT,
            CharacteristicType::Fluorescence => Fluorescence::PROMPT,
            CharacteristicType::Metallicness => Metallicness::PROMPT,
        }
    }

    pub fn str_values(&self) -> Vec<&'static str> {
        match *self {
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
}
