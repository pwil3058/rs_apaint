// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    convert::From,
    fmt,
    io::{Read, Write},
    rc::Rc,
};

use crypto_hash::{Algorithm, Hasher};

use apaint_boilerplate::BasicPaint;
use colour_math_derive::Colour;

use colour_math::{
    beigui::hue_wheel::{ColouredShape, MakeColouredShape, Shape},
    ColourBasics, LightLevel, HCV,
};

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    BasicPaintIfce, LabelText, TooltipText,
};
use std::cmp::Ordering;

#[derive(Debug, Colour, BasicPaint, Eq)]
pub struct SeriesPaint {
    colour: HCV,
    id: String,
    name: String,
    notes: String,
    finish: Finish,
    transparency: Transparency,
    permanence: Permanence,
    fluorescence: Fluorescence,
    metallicness: Metallicness,
    series_id: Rc<SeriesId>,
}

impl SeriesPaint {
    pub fn series_id(&self) -> &Rc<SeriesId> {
        &self.series_id
    }
}

impl From<(&BasicPaintSpec, &Rc<SeriesId>)> for SeriesPaint {
    fn from(spec: (&BasicPaintSpec, &Rc<SeriesId>)) -> Self {
        Self {
            colour: spec.0.colour,
            id: spec.0.id.to_string(),
            name: spec.0.name.to_string(),
            notes: spec.0.notes.to_string(),
            finish: spec.0.finish,
            transparency: spec.0.transparency,
            permanence: spec.0.permanence,
            fluorescence: spec.0.fluorescence,
            metallicness: spec.0.metallicness,
            series_id: Rc::clone(spec.1),
        }
    }
}

// TODO: think about not considering series id when testing equality and order
impl PartialEq for SeriesPaint {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id {
            self.series_id == other.series_id
        } else {
            false
        }
    }
}

impl PartialOrd for SeriesPaint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.id.cmp(&other.id) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => Some(self.series_id.cmp(&other.series_id)),
        }
    }
}

impl Ord for SeriesPaint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("shouldn't get None")
    }
}

impl TooltipText for SeriesPaint {
    fn tooltip_text(&self) -> String {
        let mut string = self.label_text();
        if let Some(notes) = self.notes() {
            string.push('\n');
            string.push_str(notes);
        };
        string.push('\n');
        string.push_str(self.series_id.series_name());
        string.push('\n');
        string.push_str(self.series_id.proprietor());

        string
    }
}

impl LabelText for SeriesPaint {
    fn label_text(&self) -> String {
        if let Some(name) = self.name() {
            format!("{}: {}", self.id, name)
        } else if let Some(notes) = self.notes() {
            format!("{}: {}", self.id, notes)
        } else {
            format!("{}: {}", self.id, self.rgb::<u8>().pango_string())
        }
    }
}

impl MakeColouredShape for SeriesPaint {
    fn coloured_shape(&self) -> ColouredShape {
        let tooltip_text = self.tooltip_text();
        ColouredShape::new(&self.colour, &self.id, &tooltip_text, Shape::Square)
    }
}

#[derive(Debug)]
pub struct SeriesPaintSeries {
    series_id: Rc<SeriesId>,
    paint_list: Vec<Rc<SeriesPaint>>,
}

impl SeriesPaintSeries {
    pub fn series_id(&self) -> &Rc<SeriesId> {
        &self.series_id
    }

    pub fn find(&self, id: &str) -> Option<&Rc<SeriesPaint>> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by_key(&id, |p| p.id()) {
            Ok(index) => self.paint_list.get(index),
            Err(_) => None,
        }
    }

    pub fn paints(&self) -> impl Iterator<Item = &Rc<SeriesPaint>> {
        self.paint_list.iter()
    }

    fn is_sorted_unique(&self) -> bool {
        for i in 1..self.paint_list.len() {
            if self.paint_list[i].id() <= self.paint_list[i - 1].id() {
                return false;
            }
        }
        true
    }
}

impl From<&SeriesPaintSeriesSpec> for SeriesPaintSeries {
    fn from(spec: &SeriesPaintSeriesSpec) -> Self {
        debug_assert!(spec.is_sorted_unique());
        let series_id = Rc::new(spec.series_id().clone());
        let mut paint_list = vec![];
        for paint_spec in spec.paints() {
            let series_paint: SeriesPaint = (paint_spec, &series_id).into();
            paint_list.push(Rc::new(series_paint));
        }
        Self {
            series_id,
            paint_list,
        }
    }
}

pub trait SeriesPaintFinder {
    fn get_series_paint(
        &self,
        paint_id: &str,
        series_id: Option<&SeriesId>,
    ) -> Result<Rc<SeriesPaint>, crate::Error>;
}

#[cfg(test)]
impl SeriesPaintFinder for SeriesPaintSeries {
    fn get_series_paint(
        &self,
        id: &str,
        _series_id: Option<&SeriesId>,
    ) -> Result<Rc<SeriesPaint>, crate::Error> {
        if let Some(paint) = self.find(id) {
            Ok(Rc::clone(paint))
        } else {
            Err(crate::Error::NotFound(id.to_string()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Colour, BasicPaint, Clone, PartialEq)]
pub struct BasicPaintSpec {
    pub colour: HCV,
    pub id: String,
    pub name: String,
    pub notes: String,
    pub finish: Finish,
    pub transparency: Transparency,
    pub permanence: Permanence,
    pub fluorescence: Fluorescence,
    pub metallicness: Metallicness,
}

impl BasicPaintSpec {
    pub fn new(colour: &impl ColourBasics, id: &str) -> Self {
        Self {
            colour: colour.hcv(),
            id: id.to_string(),
            name: String::new(),
            notes: String::new(),
            finish: Finish::default(),
            transparency: Transparency::default(),
            permanence: Permanence::default(),
            fluorescence: Fluorescence::default(),
            metallicness: Metallicness::default(),
        }
    }
}

impl MakeColouredShape for BasicPaintSpec {
    fn coloured_shape(&self) -> ColouredShape {
        let tooltip_text = if let Some(name) = self.name() {
            if let Some(notes) = self.notes() {
                format!("{}: {}\n{}", self.id, name, notes)
            } else {
                format!("{}: {}", self.id, name)
            }
        } else if let Some(notes) = self.notes() {
            format!("{}: {}", self.id, notes)
        } else {
            format!("{}: {}", self.id, self.rgb::<u8>().pango_string())
        };
        ColouredShape::new(&self.colour, &self.id, &tooltip_text, Shape::Square)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct SeriesId {
    pub(crate) proprietor: String,
    pub(crate) series_name: String,
}

impl SeriesId {
    pub fn new(series_name: &str, proprietor: &str) -> Self {
        Self {
            proprietor: proprietor.to_string(),
            series_name: series_name.to_string(),
        }
    }

    pub fn proprietor(&self) -> &str {
        &self.proprietor
    }

    pub fn series_name(&self) -> &str {
        &self.series_name
    }
}

impl fmt::Display for SeriesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:({})", self.series_name, self.proprietor)
    }
}

impl From<&Rc<SeriesId>> for SeriesId {
    fn from(sid: &Rc<SeriesId>) -> Self {
        Self {
            proprietor: sid.proprietor().to_string(),
            series_name: sid.series_name().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SeriesPaintSeriesSpec {
    pub(crate) series_id: SeriesId,
    pub(crate) paint_list: Vec<BasicPaintSpec>,
}

impl SeriesPaintSeriesSpec {
    pub fn series_id(&self) -> &SeriesId {
        &self.series_id
    }

    pub fn set_proprietor(&mut self, proprietor: &str) {
        self.series_id.proprietor = proprietor.to_string()
    }

    pub fn set_series_name(&mut self, series_name: &str) {
        self.series_id.series_name = series_name.to_string()
    }

    pub fn paints(&self) -> impl Iterator<Item = &BasicPaintSpec> {
        self.paint_list.iter()
    }

    pub fn add(&mut self, paint: &BasicPaintSpec) -> Option<BasicPaintSpec> {
        debug_assert!(self.is_sorted_unique());
        match self
            .paint_list
            .binary_search_by_key(&paint.id(), |p| p.id())
        {
            Ok(index) => {
                self.paint_list.push(paint.clone());
                let old = self.paint_list.swap_remove(index);
                debug_assert!(self.is_sorted_unique());
                Some(old)
            }
            Err(index) => {
                self.paint_list.insert(index, paint.clone());
                None
            }
        }
    }

    pub fn remove(&mut self, id: &str) -> Result<BasicPaintSpec, crate::Error> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by_key(&id, |p| p.id()) {
            Ok(index) => Ok(self.paint_list.remove(index)),
            Err(_) => Err(crate::Error::NotFound(id.to_string())),
        }
    }

    pub fn remove_all(&mut self) {
        self.paint_list.clear()
    }

    pub fn find(&self, id: &str) -> Option<&BasicPaintSpec> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by_key(&id, |p| p.id()) {
            Ok(index) => self.paint_list.get(index),
            Err(_) => None,
        }
    }

    pub fn is_sorted_unique(&self) -> bool {
        for i in 1..self.paint_list.len() {
            if self.paint_list[i].id() <= self.paint_list[i - 1].id() {
                return false;
            }
        }
        true
    }
}

impl SeriesPaintSeriesSpec {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, crate::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        let series: Self = serde_json::from_str(&string)?;
        Ok(series)
    }
}

impl SeriesPaintSeriesSpec {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<Vec<u8>, crate::Error> {
        let mut hasher = Hasher::new(Algorithm::SHA256);
        let json_text = serde_json::to_string_pretty(self)?;
        hasher.write_all(json_text.as_bytes())?;
        let digest = hasher.finish();
        writer.write_all(json_text.as_bytes())?;
        Ok(digest)
    }

    pub fn digest(&self) -> Result<Vec<u8>, crate::Error> {
        let mut hasher = Hasher::new(Algorithm::SHA256);
        let json_text = serde_json::to_string_pretty(self)?;
        hasher.write_all(json_text.as_bytes())?;
        Ok(hasher.finish())
    }
}

#[cfg(test)]
mod test {
    use crate::series::{BasicPaintSpec, SeriesPaintSeriesSpec};
    use colour_math::{HueConstants, HCV, RGB};

    #[test]
    fn save_and_recover() {
        let mut series_spec = SeriesPaintSeriesSpec::default();
        series_spec.set_proprietor("owner");
        series_spec.set_series_name("series name");
        assert!(series_spec.paints().next().is_none());
        series_spec.add(&BasicPaintSpec::new(&RGB::<f64>::RED, "red"));
        series_spec.add(&BasicPaintSpec::new(&HCV::YELLOW, "yellow"));
        let mut buffer: Vec<u8> = vec![];
        let _digest = series_spec.write(&mut buffer);
        let read_spec = SeriesPaintSeriesSpec::read(&mut &buffer[..]).unwrap();
        assert_eq!(series_spec.series_id(), read_spec.series_id());
        assert_eq!(series_spec.paint_list.len(), read_spec.paint_list.len());
        for (pspec1, pspec2) in series_spec.paints().zip(read_spec.paints()) {
            assert_eq!(*pspec1, *pspec2);
        }
    }
}
