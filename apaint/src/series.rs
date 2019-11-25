// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::marker::PhantomData;

use crate::BasicPaintIfce;
use colour_math::ColourComponent;

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
