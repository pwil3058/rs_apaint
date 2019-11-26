// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{io::Read, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};

use colour_math::ColourComponent;

use crate::BasicPaintIfce;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Default)]
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

    pub fn remove(&mut self, id: &str) -> P {
        debug_assert!(self.is_sorted_unique());
        match self.paint_list.binary_search_by(|p| p.id().cmp(id)) {
            Ok(index) => self.paint_list.remove(index),
            Err(_) => panic!("{}: id not found", id),
        }
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
    P: BasicPaintIfce<F> + Serialize + DeserializeOwned + Clone,
{
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, crate::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        let series: Self = serde_json::from_str(&string)?;
        Ok(series)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), crate::Error> {
        let json_text = serde_json::to_string_pretty(self)?;
        writer.write_all(json_text.as_bytes())?;
        Ok(())
    }
}
