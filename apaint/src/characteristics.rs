// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

//! Types to describe paint characteristics that cannot be derived from their colour.

use std::str::FromStr;

pub trait CharacteristicIfce: FromStr + PartialEq + PartialOrd {
    const NAME: &'static str;
    const PROMPT: &'static str;

    fn abbrev(&self) -> &'static str;
    fn full(&self) -> &'static str;
}

/// Finish.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Finish {
    Gloss,
    SemiGloss,
    SemiFlat,
    Flat,
}

impl CharacteristicIfce for Finish {
    const NAME: &'static str = "Finish";
    const PROMPT: &'static str = "Finish:";

    fn abbrev(&self) -> &'static str {
        match *self {
            Finish::Gloss => "G",
            Finish::SemiGloss => "SG",
            Finish::SemiFlat => "SF",
            Finish::Flat => "F",
        }
    }

    fn full(&self) -> &'static str {
        match *self {
            Finish::Gloss => "Gloss",
            Finish::SemiGloss => "Semi-gloss",
            Finish::SemiFlat => "Semi-flat",
            Finish::Flat => "Flat",
        }
    }
}

impl FromStr for Finish {
    type Err = String;

    fn from_str(string: &str) -> Result<Finish, String> {
        match string {
            "G" | "Gloss" => Ok(Finish::Gloss),
            "SG" | "Semi-gloss" => Ok(Finish::SemiGloss),
            "SF" | "Semi-flat" => Ok(Finish::SemiFlat),
            "F" | "Flat" => Ok(Finish::Flat),
            _ => Err(format!("\"{}\": Malformed 'Finish' value string", string)),
        }
    }
}
