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

#[derive(Debug)]
pub enum CharacteristicTypes {
    Finish,
    Transparency,
    Permanence,
    Fluorescence,
    Metalicness,
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
