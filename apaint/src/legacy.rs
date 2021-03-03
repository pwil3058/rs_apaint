// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{io::Read, str::FromStr};

use regex::Regex;

use lazy_static::lazy_static;

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    series::{BasicPaintSpec, SeriesPaintSeriesSpec},
};

pub mod legacy_series;

lazy_static! {
    static ref HEADER_RE: Regex = Regex::new(r"^\w+:\s*(.*)$").expect("programmer error");
    static ref PAINT_RE: Regex = Regex::new(
        r#"^(?P<ptype>\w+)\((name=)?"(?P<name>.+)", rgb=(?P<rgb>RGB(16)?\([^)]+\))(?P<characteristics>(?:, [^n]\w+="\w+")*)(, notes="(?P<notes>.*)")?\)$"#
    ).expect("programmer error");
    static ref CHARACTERISTIC_RE: Regex = Regex::new(r###"(\w+)="(\w+)""###).expect("programmer error");
}

fn extract_header_value(line: Option<&str>) -> Result<String, crate::Error> {
    if let Some(line) = line {
        if let Some(cap) = HEADER_RE.captures(line) {
            return Ok(cap[1].to_string());
        }
    };
    Err(crate::Error::NotAValidLegacySpec)
}

fn extract_paint_spec(line: &str) -> Result<BasicPaintSpec, crate::Error> {
    use crate::Error::NotAValidLegacySpec;
    if let Some(cap) = PAINT_RE.captures(line) {
        let name = cap.name("name").ok_or(NotAValidLegacySpec)?.as_str();
        let rgb_str = cap.name("rgb").ok_or(NotAValidLegacySpec)?.as_str();
        let rgb = colour_math::RGB::<u16>::from_str(rgb_str).map_err(|_| NotAValidLegacySpec)?;
        let mut bps = BasicPaintSpec::new(&rgb, name);
        bps.name = name.to_string();
        bps.notes = cap
            .name("notes")
            .ok_or(NotAValidLegacySpec)?
            .as_str()
            .to_string();
        let characteristics_str = cap
            .name("characteristics")
            .ok_or(NotAValidLegacySpec)?
            .as_str();
        for m in CHARACTERISTIC_RE.find_iter(characteristics_str) {
            let c = CHARACTERISTIC_RE
                .captures(m.as_str())
                .ok_or(NotAValidLegacySpec)?;
            match &c[1] {
                "finish" => {
                    bps.finish = Finish::from_str(&c[2]).map_err(|_| NotAValidLegacySpec)?
                }
                "fluorescence" => {
                    bps.fluorescence =
                        Fluorescence::from_str(&c[2]).map_err(|_| NotAValidLegacySpec)?
                }
                "permanence" => {
                    bps.permanence = Permanence::from_str(&c[2]).map_err(|_| NotAValidLegacySpec)?
                }
                "metallicness" | "metallic" => {
                    bps.metallicness =
                        Metallicness::from_str(&c[2]).map_err(|_| NotAValidLegacySpec)?
                }
                "transparency" => {
                    bps.transparency =
                        Transparency::from_str(&c[2]).map_err(|_| NotAValidLegacySpec)?
                }
                _ => return Err(NotAValidLegacySpec),
            }
        }
        Ok(bps)
    } else {
        Err(crate::Error::NotAValidLegacySpec)
    }
}

pub fn extract_legacy_paint_series_spec(
    string: &str,
) -> Result<SeriesPaintSeriesSpec, crate::Error> {
    let mut lines = string.lines();
    let mut spec = SeriesPaintSeriesSpec::default();
    let series_name = extract_header_value(lines.next())?;
    spec.set_series_name(&series_name);
    let proprieter = extract_header_value(lines.next())?;
    spec.set_proprietor(&proprieter);
    for line in lines {
        let paint_spec = extract_paint_spec(line)?;
        spec.add(&paint_spec);
    }
    Ok(spec)
}

pub fn read_legacy_paint_series_spec<R: Read>(
    reader: &mut R,
) -> Result<SeriesPaintSeriesSpec, crate::Error> {
    let mut string = String::new();
    reader.read_to_string(&mut string)?;
    let spec = extract_legacy_paint_series_spec(&string)?;
    Ok(spec)
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_TEXT: &str = r###"Standard: Federal Standard 595C: Artificial Flat (<http://www.colorserver.net/>)
Sponsor: U.S. Government
PaintSpec(name="FS30032", rgb=RGB16(red=0x2821, green=0x0221, blue=0x012D), finish="F", transparency="O", fluorescence="NF", metallic="NM", notes="")
PaintSpec(name="FS30040", rgb=RGB16(red=0x4200, green=0x3F00, blue=0x3800), finish="F", transparency="O", fluorescence="NF", metallic="NM", notes="")
PaintSpec(name="FS30045", rgb=RGB16(red=0x4DEA, green=0x3CEA, blue=0x35EA), finish="F", transparency="O", fluorescence="NF", metallic="NM", notes="")
PaintSpec(name="FS30049", rgb=RGB16(red=0x37E5, green=0x04E5, blue=0x03E5), finish="F", transparency="O", fluorescence="NF", metallic="NM", notes="")
PaintSpec(name="FS30051", rgb=RGB16(red=0x50D8, green=0x42D8, blue=0x39D8), finish="F", transparency="O", fluorescence="NF", metallic="NM", notes="Leather Brown")
PaintSpec(name="FS30055", rgb=RGB16(red=0x5900, green=0x2D00, blue=0x1000), finish="F", transparency="O", fluorescence="NF", metallic="NM", notes="")
PaintSpec(name="FS30059", rgb=RGB16(red=0x4700, green=0x3300, blue=0x2800), finish="F", transparency="O", fluorescence="NF", metallic="NM", notes="")
"###;

    #[test]
    fn read_legacy_files() {
        let mut lines = TEST_TEXT.lines();
        let line = lines.next().unwrap();
        println!("line: {:?}", line);
        let cap = HEADER_RE.captures(line);
        println!("cap: {:?}", cap);
        let cap = cap.unwrap();
        assert_eq!(
            &cap[1],
            r###"Federal Standard 595C: Artificial Flat (<http://www.colorserver.net/>)"###
        );
        let line = lines.next().unwrap();
        println!("line: {:?}", line);
        let cap = HEADER_RE.captures(line);
        println!("cap: {:?}", cap);
        let cap = cap.unwrap();
        assert_eq!(&cap[1], r###"U.S. Government"###);
        for line in lines {
            println!("line: {:?}", line);
            let cap = PAINT_RE.captures(line);
            println!("cap: {:?}", cap);
            assert!(cap.is_some());
        }
    }

    #[test]
    fn extract_legacy_series() {
        assert!(extract_legacy_paint_series_spec(&TEST_TEXT).is_ok());
    }
}
