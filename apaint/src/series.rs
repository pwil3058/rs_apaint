// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    convert::From,
    fmt,
    io::{Read, Write},
    rc::Rc,
};

use crypto_hash::{Algorithm, Hasher};
use serde::{de::DeserializeOwned, Serialize};

use apaint_boilerplate::{BasicPaint, Colour};

use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    hue_wheel::{ColouredShape, MakeColouredShape, Shape, ShapeConsts},
    BasicPaintIfce, LabelText, TooltipText,
};
use std::cmp::Ordering;

#[derive(Debug, Colour, BasicPaint)]
pub struct SeriesPaint<F: ColourComponent> {
    rgb: RGB<F>,
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

impl<F: ColourComponent> SeriesPaint<F> {
    pub fn series_id(&self) -> &Rc<SeriesId> {
        &self.series_id
    }
}

impl<F: ColourComponent> From<(&BasicPaintSpec<F>, &Rc<SeriesId>)> for SeriesPaint<F> {
    fn from(spec: (&BasicPaintSpec<F>, &Rc<SeriesId>)) -> Self {
        Self {
            rgb: spec.0.rgb,
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
impl<F: ColourComponent> PartialEq for SeriesPaint<F> {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id {
            self.series_id == other.series_id
        } else {
            false
        }
    }
}

impl<F: ColourComponent> Eq for SeriesPaint<F> {}

impl<F: ColourComponent> PartialOrd for SeriesPaint<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.id.cmp(&other.id) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => Some(self.series_id.cmp(&other.series_id)),
        }
    }
}

impl<F: ColourComponent> Ord for SeriesPaint<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<F: ColourComponent> TooltipText for SeriesPaint<F> {
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

impl<F: ColourComponent> LabelText for SeriesPaint<F> {
    fn label_text(&self) -> String {
        if let Some(name) = self.name() {
            format!("{}: {}", self.id, name)
        } else if let Some(notes) = self.notes() {
            format!("{}: {}", self.id, notes)
        } else {
            format!("{}: {}", self.id, self.rgb().pango_string())
        }
    }
}

impl<F: ColourComponent + ShapeConsts> MakeColouredShape<F> for SeriesPaint<F> {
    fn coloured_shape(&self) -> ColouredShape<F> {
        let tooltip_text = self.tooltip_text();
        ColouredShape::new(self.rgb, &self.id, &tooltip_text, Shape::Square)
    }
}

#[derive(Debug)]
pub struct SeriesPaintSeries<F>
where
    F: ColourComponent,
{
    series_id: Rc<SeriesId>,
    paint_list: Vec<Rc<SeriesPaint<F>>>,
}

impl<F> SeriesPaintSeries<F>
where
    F: ColourComponent,
{
    pub fn series_id(&self) -> &Rc<SeriesId> {
        &self.series_id
    }

    pub fn find(&self, id: &str) -> Option<&Rc<SeriesPaint<F>>> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by_key(&id, |p| p.id()) {
            Ok(index) => self.paint_list.get(index),
            Err(_) => None,
        }
    }

    pub fn paints(&self) -> impl Iterator<Item = &Rc<SeriesPaint<F>>> {
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

impl<F: ColourComponent> From<&SeriesPaintSeriesSpec<F>> for SeriesPaintSeries<F> {
    fn from(spec: &SeriesPaintSeriesSpec<F>) -> Self {
        debug_assert!(spec.is_sorted_unique());
        let series_id = Rc::new(spec.series_id().clone());
        let mut paint_list = vec![];
        for paint_spec in spec.paints() {
            let series_paint: SeriesPaint<F> = (paint_spec, &series_id).into();
            paint_list.push(Rc::new(series_paint));
        }
        Self {
            series_id,
            paint_list,
        }
    }
}

pub trait SeriesPaintFinder<F: ColourComponent> {
    fn get_series_paint(
        &self,
        paint_id: &str,
        series_id: Option<&SeriesId>,
    ) -> Result<Rc<SeriesPaint<F>>, crate::Error>;
}

#[derive(Debug, Serialize, Deserialize, Colour, BasicPaint, Clone, PartialEq)]
pub struct BasicPaintSpec<F: ColourComponent> {
    pub rgb: RGB<F>,
    pub id: String,
    pub name: String,
    pub notes: String,
    pub finish: Finish,
    pub transparency: Transparency,
    pub permanence: Permanence,
    pub fluorescence: Fluorescence,
    pub metallicness: Metallicness,
}

impl<F: ColourComponent> BasicPaintSpec<F> {
    pub fn new(rgb: RGB<F>, id: &str) -> Self {
        Self {
            rgb,
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

impl<F: ColourComponent + ShapeConsts> MakeColouredShape<F> for BasicPaintSpec<F> {
    fn coloured_shape(&self) -> ColouredShape<F> {
        let tooltip_text = if let Some(name) = self.name() {
            if let Some(notes) = self.notes() {
                format!("{}: {}\n{}", self.id, name, notes)
            } else {
                format!("{}: {}", self.id, name)
            }
        } else if let Some(notes) = self.notes() {
            format!("{}: {}", self.id, notes)
        } else {
            format!("{}: {}", self.id, self.rgb().pango_string())
        };
        ColouredShape::new(self.rgb, &self.id, &tooltip_text, Shape::Square)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct SeriesId {
    proprietor: String,
    series_name: String,
}

impl SeriesId {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct SeriesPaintSeriesSpec<F>
where
    F: ColourComponent,
{
    series_id: SeriesId,
    paint_list: Vec<BasicPaintSpec<F>>,
}

impl<F> std::default::Default for SeriesPaintSeriesSpec<F>
where
    F: ColourComponent,
{
    fn default() -> Self {
        Self {
            series_id: SeriesId::default(),
            paint_list: Vec::new(),
        }
    }
}

impl<F> SeriesPaintSeriesSpec<F>
where
    F: ColourComponent,
{
    pub fn series_id(&self) -> &SeriesId {
        &self.series_id
    }

    pub fn set_proprietor(&mut self, proprietor: &str) {
        self.series_id.proprietor = proprietor.to_string()
    }

    pub fn set_series_name(&mut self, series_name: &str) {
        self.series_id.series_name = series_name.to_string()
    }

    pub fn paints(&self) -> impl Iterator<Item = &BasicPaintSpec<F>> {
        self.paint_list.iter()
    }

    pub fn add(&mut self, paint: &BasicPaintSpec<F>) -> Option<BasicPaintSpec<F>> {
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

    pub fn remove(&mut self, id: &str) -> Result<BasicPaintSpec<F>, crate::Error> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by_key(&id, |p| p.id()) {
            Ok(index) => Ok(self.paint_list.remove(index)),
            Err(_) => Err(crate::Error::NotFound(id.to_string())),
        }
    }

    pub fn remove_all(&mut self) {
        self.paint_list.clear()
    }

    pub fn find(&self, id: &str) -> Option<&BasicPaintSpec<F>> {
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

impl<'de, F> SeriesPaintSeriesSpec<F>
where
    F: ColourComponent + DeserializeOwned,
{
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, crate::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        let series: Self = serde_json::from_str(&string)?;
        Ok(series)
    }
}

impl<'de, F> SeriesPaintSeriesSpec<F>
where
    F: ColourComponent + Serialize,
{
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
    use colour_math::RGB;

    #[test]
    fn save_and_recover() {
        let mut series_spec = SeriesPaintSeriesSpec::<f64>::default();
        series_spec.set_proprietor("owner");
        series_spec.set_series_name("series name");
        assert!(series_spec.paints().next().is_none());
        series_spec.add(&BasicPaintSpec::new(RGB::<f64>::RED, "red"));
        series_spec.add(&BasicPaintSpec::new(RGB::<f64>::YELLOW, "yellow"));
        let mut buffer: Vec<u8> = vec![];
        let _digest = series_spec.write(&mut buffer);
        let read_spec = SeriesPaintSeriesSpec::<f64>::read(&mut &buffer[..]).unwrap();
        assert_eq!(series_spec.series_id(), read_spec.series_id());
        assert_eq!(series_spec.paint_list.len(), read_spec.paint_list.len());
        for (pspec1, pspec2) in series_spec.paints().zip(read_spec.paints()) {
            assert_eq!(*pspec1, *pspec2);
        }
    }
}
