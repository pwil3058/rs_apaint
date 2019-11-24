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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PaintSeries<F, P>
where
    F: ColourComponent,
    P: BasicPaintIfce<F>,
{
    series_id: SeriesId,
    paint_list: Vec<P>,
    phantom_data: PhantomData<F>,
}

impl<F, P> PaintSeries<F, P>
where
    F: ColourComponent,
    P: BasicPaintIfce<F>,
{
    pub fn series_id(&self) -> &SeriesId {
        &self.series_id
    }

    pub fn paints(&self) -> impl Iterator<Item = &P> {
        self.paint_list.iter()
    }
}
