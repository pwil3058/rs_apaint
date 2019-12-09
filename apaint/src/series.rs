// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{convert::From, fmt, rc::Rc};

use apaint_boilerplate::{BasicPaint, Colour};

use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

use crate::hue_wheel::{MakeColouredShape, ShapeConsts};
use crate::spec::{BasicPaintSeriesSpec, BasicPaintSpec};
use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    hue_wheel::{ColouredShape, Shape},
    BasicPaintIfce, LabelText, TooltipText,
};

#[derive(Serialize, Deserialize, Debug, Default, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct SeriesId {
    pub(crate) proprietor: String,
    pub(crate) series_name: String,
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
    pub fn seried_id(&self) -> &Rc<SeriesId> {
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
        match self.paint_list.binary_search_by(|p| p.id().cmp(id)) {
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

impl<F: ColourComponent> From<&BasicPaintSeriesSpec<F>> for SeriesPaintSeries<F> {
    fn from(spec: &BasicPaintSeriesSpec<F>) -> Self {
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
