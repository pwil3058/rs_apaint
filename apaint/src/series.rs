// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    fmt,
    io::{Read, Write},
    marker::PhantomData,
    rc::Rc,
};

use crypto_hash::{Algorithm, Hasher};
use serde::{de::DeserializeOwned, Serialize};

use apaint_boilerplate::Colour;

use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

use crate::hue_wheel::{MakeColouredShape, ShapeConsts};
use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    hue_wheel::{ColouredShape, Shape},
    BasicPaintIfce, LabelText, TooltipText,
};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct PaintSeries<F, P>
where
    F: ColourComponent,
    P: BasicPaintIfce<F>,
{
    series_id: SeriesId,
    paint_list: Vec<P>,
    phantom_data: PhantomData<F>,
}

impl<F, P> std::default::Default for PaintSeries<F, P>
where
    F: ColourComponent,
    P: BasicPaintIfce<F>,
{
    fn default() -> Self {
        Self {
            series_id: SeriesId::default(),
            paint_list: Vec::new(),
            phantom_data: PhantomData,
        }
    }
}

impl<F, P> PaintSeries<F, P>
where
    F: ColourComponent,
    P: BasicPaintIfce<F> + Clone,
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

    pub fn paints(&self) -> impl Iterator<Item = &P> {
        self.paint_list.iter()
    }

    pub fn add(&mut self, paint: &P) -> Option<P> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by(|p| p.id().cmp(paint.id())) {
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

    pub fn remove(&mut self, id: &str) -> Result<P, crate::Error> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by(|p| p.id().cmp(id)) {
            Ok(index) => Ok(self.paint_list.remove(index)),
            Err(_) => Err(crate::Error::NotFound(id.to_string())),
        }
    }

    pub fn remove_all(&mut self) {
        self.paint_list.clear()
    }

    pub fn find(&self, id: &str) -> Option<&P> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by(|p| p.id().cmp(id)) {
            Ok(index) => self.paint_list.get(index),
            Err(_) => None,
        }
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

impl<'de, F, P> PaintSeries<F, P>
where
    F: ColourComponent,
    P: BasicPaintIfce<F> + DeserializeOwned + Clone,
{
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, crate::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        let series: Self = serde_json::from_str(&string)?;
        Ok(series)
    }
}

impl<'de, F, P> PaintSeries<F, P>
where
    F: ColourComponent,
    P: BasicPaintIfce<F> + Serialize + Clone,
{
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), crate::Error> {
        let json_text = serde_json::to_string_pretty(self)?;
        writer.write_all(json_text.as_bytes())?;
        Ok(())
    }

    pub fn digest(&self) -> Result<Vec<u8>, crate::Error> {
        let mut hasher = Hasher::new(Algorithm::SHA256);
        let json_text = serde_json::to_string(self)?;
        hasher.write_all(json_text.as_bytes())?;
        Ok(hasher.finish())
    }
}

#[derive(Debug, Colour, Clone)]
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
    pub fn seried_id(&self) -> &Rc<SeriesId> {
        &self.series_id
    }
}

impl<F: ColourComponent> BasicPaintIfce<F> for SeriesPaint<F> {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> Option<&str> {
        if self.name.len() == 0 {
            None
        } else {
            Some(&self.name)
        }
    }

    fn notes(&self) -> Option<&str> {
        if self.notes.len() == 0 {
            None
        } else {
            Some(&self.notes)
        }
    }

    fn finish(&self) -> Finish {
        self.finish
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn fluorescence(&self) -> Fluorescence {
        self.fluorescence
    }

    fn permanence(&self) -> Permanence {
        self.permanence
    }

    fn metallicness(&self) -> Metallicness {
        self.metallicness
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
