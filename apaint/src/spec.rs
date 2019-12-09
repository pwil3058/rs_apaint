// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    io::{Read, Write},
    string::ToString,
};

use crypto_hash::{Algorithm, Hasher};
use serde::{de::DeserializeOwned, Serialize};

use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

use apaint_boilerplate::{BasicPaint, Colour};

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    hue_wheel::{ColouredShape, MakeColouredShape, Shape, ShapeConsts},
    series::SeriesId,
    BasicPaintIfce,
};

#[derive(Debug, Serialize, Deserialize, Colour, BasicPaint, Clone)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicPaintSeriesSpec<F>
where
    F: ColourComponent,
{
    series_id: SeriesId,
    paint_list: Vec<BasicPaintSpec<F>>,
}

impl<F> std::default::Default for BasicPaintSeriesSpec<F>
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

impl<F> BasicPaintSeriesSpec<F>
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

    pub fn remove(&mut self, id: &str) -> Result<BasicPaintSpec<F>, crate::Error> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by(|p| p.id().cmp(id)) {
            Ok(index) => Ok(self.paint_list.remove(index)),
            Err(_) => Err(crate::Error::NotFound(id.to_string())),
        }
    }

    pub fn remove_all(&mut self) {
        self.paint_list.clear()
    }

    pub fn find(&self, id: &str) -> Option<&BasicPaintSpec<F>> {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by(|p| p.id().cmp(id)) {
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

impl<'de, F> BasicPaintSeriesSpec<F>
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

impl<'de, F> BasicPaintSeriesSpec<F>
where
    F: ColourComponent + Serialize,
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
