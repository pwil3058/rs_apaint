// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use regex::Regex;

use lazy_static::lazy_static;

lazy_static! {
    static ref HEADER_RE: Regex = Regex::new(r"^\w+:\s*(.*)$").expect("programmer error");
    static ref PAINT_RE: Regex = Regex::new(
        r#"^(?P<ptype>\w+)\((name=)?"(?P<name>.+)", rgb=(?P<rgb>RGB(16)?\([^)]+\))(?P<characteristics>(?:, \w+="\w+")*)(, notes="(?P<notes>.*)")?\)$"#
    ).expect("programmer error");
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
        let line = lines.next().unwrap();
        println!("line: {:?}", line);
        let cap = PAINT_RE.captures(line);
        println!("cap: {:?}", cap);
        assert!(cap.is_some());
    }
}
