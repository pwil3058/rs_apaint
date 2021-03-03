// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{convert::From, io::Read};

use serde::de::DeserializeOwned;

use apaint_boilerplate::BasicPaint;
use colour_math_derive::Colour;

use colour_math::{ColourBasics, LightLevel, RGB};

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    series::{BasicPaintSpec, SeriesId, SeriesPaintSeriesSpec},
    BasicPaintIfce,
};

#[derive(Debug, Serialize, Deserialize, Colour, BasicPaint, Clone, PartialEq)]
pub struct BasicPaintSpec00<F: LightLevel> {
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

impl<F: LightLevel> From<&BasicPaintSpec00<F>> for BasicPaintSpec {
    fn from(paint00: &BasicPaintSpec00<F>) -> Self {
        Self {
            colour: paint00.hcv(),
            id: paint00.id().to_string(),
            name: if let Some(name) = paint00.name() {
                name.to_string()
            } else {
                "".to_string()
            },
            notes: if let Some(notes) = paint00.notes() {
                notes.to_string()
            } else {
                "".to_string()
            },
            finish: paint00.finish(),
            transparency: paint00.transparency(),
            permanence: paint00.permanence(),
            fluorescence: paint00.fluorescence(),
            metallicness: paint00.metallicness(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SeriesPaintSeriesSpec00<F>
where
    F: LightLevel,
{
    series_id: SeriesId,
    paint_list: Vec<BasicPaintSpec00<F>>,
}

impl<F: LightLevel> From<SeriesPaintSeriesSpec00<F>> for SeriesPaintSeriesSpec {
    fn from(spec: SeriesPaintSeriesSpec00<F>) -> Self {
        let series_id: SeriesId = SeriesId::from(spec.series_id.clone());
        let mut paint_list: Vec<BasicPaintSpec> = vec![];
        for paint in &spec.paint_list {
            paint_list.push(paint.into());
        }
        Self {
            series_id,
            paint_list,
        }
    }
}

impl<'de, F> SeriesPaintSeriesSpec00<F>
where
    F: LightLevel + DeserializeOwned,
{
    pub fn read<R: Read>(reader: &mut R) -> Result<SeriesPaintSeriesSpec, crate::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        let series: Self = serde_json::from_str(&string)?;
        Ok(series.into())
    }
}

#[cfg(test)]
mod test {
    use std::{convert::From, io::Write};

    use crypto_hash::{Algorithm, Hasher};
    use serde::Serialize;

    use colour_math::{HueConstants, LightLevel, RGB};

    use crate::legacy::legacy_series::{BasicPaintSpec00, SeriesPaintSeriesSpec00};
    use crate::series::{BasicPaintSpec, SeriesId};
    use crate::BasicPaintIfce;

    use crate::characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency};

    impl<F: LightLevel> BasicPaintSpec00<F> {
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

    impl<F> std::default::Default for SeriesPaintSeriesSpec00<F>
    where
        F: LightLevel,
    {
        fn default() -> Self {
            Self {
                series_id: SeriesId::default(),
                paint_list: Vec::new(),
            }
        }
    }

    impl<F> SeriesPaintSeriesSpec00<F>
    where
        F: LightLevel,
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

        pub fn paints(&self) -> impl Iterator<Item = &BasicPaintSpec00<F>> {
            self.paint_list.iter()
        }

        pub fn add(&mut self, paint: &BasicPaintSpec00<F>) -> Option<BasicPaintSpec00<F>> {
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

        pub fn remove(&mut self, id: &str) -> Result<BasicPaintSpec00<F>, crate::Error> {
            debug_assert!(self.is_sorted_unique());
            match self.paint_list.binary_search_by_key(&id, |p| p.id()) {
                Ok(index) => Ok(self.paint_list.remove(index)),
                Err(_) => Err(crate::Error::NotFound(id.to_string())),
            }
        }

        pub fn remove_all(&mut self) {
            self.paint_list.clear()
        }

        pub fn find(&self, id: &str) -> Option<&BasicPaintSpec00<F>> {
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

    impl<'de, F> SeriesPaintSeriesSpec00<F>
    where
        F: LightLevel + Serialize,
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

    #[test]
    fn save_and_recover() {
        let mut series_spec = SeriesPaintSeriesSpec00::<f64>::default();
        series_spec.set_proprietor("owner");
        series_spec.set_series_name("series name");
        assert!(series_spec.paints().next().is_none());
        series_spec.add(&BasicPaintSpec00::new(RGB::<f64>::RED, "red"));
        series_spec.add(&BasicPaintSpec00::new(RGB::<f64>::YELLOW, "yellow"));
        let mut buffer: Vec<u8> = vec![];
        let _digest = series_spec.write(&mut buffer);
        let read_spec = SeriesPaintSeriesSpec00::<f64>::read(&mut &buffer[..]).unwrap();
        assert_eq!(series_spec.series_id(), read_spec.series_id());
        assert_eq!(series_spec.paint_list.len(), read_spec.paint_list.len());
        for (pspec1, pspec2) in series_spec.paints().zip(read_spec.paints()) {
            assert_eq!(BasicPaintSpec::from(pspec1), *pspec2);
        }
    }
}
