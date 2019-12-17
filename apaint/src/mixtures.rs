// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use num::Integer;

use colour_math::{ColourComponent, ColourInterface, Degrees, Hue, ScalarAttribute, RGB};

use apaint_boilerplate::{BasicPaint, Colour};

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    hue_wheel::{ColouredShape, MakeColouredShape, Shape, ShapeConsts},
    series::SeriesPaint,
    BasicPaintIfce, LabelText, TooltipText,
};

#[derive(Debug, Colour)]
pub struct MixedPaint<F: ColourComponent> {
    rgb: RGB<F>,
    targeted_rgb: Option<RGB<F>>,
    id: String,
    name: String,
    notes: String,
    finish: f64,
    transparency: f64,
    permanence: f64,
    fluorescence: f64,
    metallicness: f64,
    components: Vec<(Paint<F>, u64)>,
}

impl<F: ColourComponent + ShapeConsts> MixedPaint<F> {
    pub fn targeted_rgb(&self) -> Option<&RGB<F>> {
        if let Some(ref rgb) = self.targeted_rgb {
            Some(rgb)
        } else {
            None
        }
    }

    pub fn targeted_rgb_shape(&self) -> ColouredShape<F> {
        let tooltip_text = format!("Target for: {}", self.tooltip_text());
        let id = self.targeted_rgb_id();
        ColouredShape::new(
            self.targeted_rgb.unwrap(),
            &id,
            &tooltip_text,
            Shape::Circle,
        )
    }

    pub fn targeted_rgb_id(&self) -> String {
        format!("TARGET({})", self.id)
    }
}

impl<F: ColourComponent> BasicPaintIfce<F> for MixedPaint<F> {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> Option<&str> {
        if self.name.len() > 0 {
            Some(&self.name)
        } else {
            None
        }
    }

    fn notes(&self) -> Option<&str> {
        if self.notes.len() > 0 {
            Some(&self.notes)
        } else {
            None
        }
    }

    fn finish(&self) -> Finish {
        self.finish.into()
    }

    fn transparency(&self) -> Transparency {
        self.transparency.into()
    }

    fn fluorescence(&self) -> Fluorescence {
        self.fluorescence.into()
    }

    fn permanence(&self) -> Permanence {
        self.permanence.into()
    }

    fn metallicness(&self) -> Metallicness {
        self.metallicness.into()
    }
}

impl<F: ColourComponent> TooltipText for MixedPaint<F> {
    fn tooltip_text(&self) -> String {
        let mut string = self.label_text();
        if let Some(notes) = self.notes() {
            string.push('\n');
            string.push_str(notes);
        };

        string
    }
}

impl<F: ColourComponent> LabelText for MixedPaint<F> {
    fn label_text(&self) -> String {
        if let Some(name) = self.name() {
            format!("Mix {}: {}", self.id, name)
        } else if let Some(notes) = self.notes() {
            format!("Mix {}: {}", self.id, notes)
        } else {
            format!("Mix {}: {}", self.id, self.rgb().pango_string())
        }
    }
}

impl<F: ColourComponent + ShapeConsts> MakeColouredShape<F> for MixedPaint<F> {
    fn coloured_shape(&self) -> ColouredShape<F> {
        let tooltip_text = self.tooltip_text();
        ColouredShape::new(self.rgb, &self.id, &tooltip_text, Shape::Diamond)
    }
}

#[derive(Debug)]
pub struct MixingSession<F: ColourComponent> {
    notes: String,
    mixtures: Vec<Rc<MixedPaint<F>>>,
}

impl<F: ColourComponent> MixingSession<F> {
    pub fn new() -> Self {
        Self {
            notes: String::new(),
            mixtures: vec![],
        }
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }

    pub fn set_notes(&mut self, notes: &str) {
        self.notes = notes.to_string()
    }

    pub fn mixtures(&self) -> impl Iterator<Item = &Rc<MixedPaint<F>>> {
        self.mixtures.iter()
    }

    pub fn add_mixture(&mut self, mixture: &Rc<MixedPaint<F>>) -> Option<Rc<MixedPaint<F>>> {
        debug_assert!(self.is_sorted_unique());
        match self.mixtures.binary_search_by(|p| p.id().cmp(mixture.id())) {
            Ok(index) => {
                self.mixtures.push(Rc::clone(mixture));
                let old = self.mixtures.swap_remove(index);
                debug_assert!(self.is_sorted_unique());
                Some(old)
            }
            Err(index) => {
                self.mixtures.insert(index, Rc::clone(mixture));
                None
            }
        }
    }

    pub fn is_sorted_unique(&self) -> bool {
        for i in 1..self.mixtures.len() {
            if self.mixtures[i].id() <= self.mixtures[i - 1].id() {
                return false;
            }
        }
        true
    }
}

#[derive(Debug)]
pub struct MixedPaintBuilder<F: ColourComponent> {
    id: String,
    name: String,
    notes: String,
    series_components: Vec<(Rc<SeriesPaint<F>>, u64)>,
    mixture_components: Vec<(Rc<MixedPaint<F>>, u64)>,
    targeted_rgb: Option<RGB<F>>,
}

impl<F: ColourComponent> MixedPaintBuilder<F> {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: String::new(),
            notes: String::new(),
            series_components: vec![],
            mixture_components: vec![],
            targeted_rgb: None,
        }
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn notes(&mut self, notes: &str) -> &mut Self {
        self.notes = notes.to_string();
        self
    }

    pub fn targeted_rgb(&mut self, rgb: &RGB<F>) -> &mut Self {
        self.targeted_rgb = Some(*rgb);
        self
    }

    pub fn series_paint_components(
        &mut self,
        components: Vec<(Rc<SeriesPaint<F>>, u64)>,
    ) -> &mut Self {
        self.series_components = components;
        self
    }

    pub fn build(&self) -> Rc<MixedPaint<F>> {
        debug_assert!((self.series_components.len() + self.mixture_components.len()) > 0);
        let mut gcd: u64 = 0;
        for (_, parts) in self.series_components.iter() {
            debug_assert!(*parts > 0);
            gcd = gcd.gcd(parts);
        }
        for (_, parts) in self.mixture_components.iter() {
            debug_assert!(*parts > 0);
            gcd = gcd.gcd(parts);
        }
        debug_assert!(gcd > 0);
        let mut components = vec![];
        let mut total_adjusted_parts: u64 = 0;
        let mut rgb_sum: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        let mut finish: f64 = 0.0;
        let mut transparency: f64 = 0.0;
        let mut permanence: f64 = 0.0;
        let mut fluorescence: f64 = 0.0;
        let mut metallicness: f64 = 0.0;
        for (paint, parts) in self.series_components.iter() {
            let adjusted_parts = parts / gcd;
            total_adjusted_parts += adjusted_parts;
            let rgb = paint.rgb();
            for i in 0..3 {
                rgb_sum[i] +=
                    rgb[i as u8] * F::from_u64(adjusted_parts).expect("no problems expected");
            }
            let fap = adjusted_parts as f64;
            finish += fap * f64::from(paint.finish());
            transparency += fap * f64::from(paint.transparency());
            permanence += fap * f64::from(paint.permanence());
            fluorescence += fap * f64::from(paint.fluorescence());
            metallicness += fap * f64::from(paint.metallicness());
            components.push((Paint::Series(Rc::clone(paint)), adjusted_parts));
        }
        for (paint, parts) in self.mixture_components.iter() {
            let adjusted_parts = parts / gcd;
            total_adjusted_parts += adjusted_parts;
            let rgb = paint.rgb();
            for i in 0..3 {
                rgb_sum[i] +=
                    rgb[i as u8] * F::from_u64(adjusted_parts).expect("no problems expected");
            }
            let fap = adjusted_parts as f64;
            finish += fap * paint.finish;
            transparency += fap * paint.transparency;
            permanence += fap * paint.permanence;
            fluorescence += fap * paint.fluorescence;
            metallicness += fap * paint.metallicness;
            components.push((Paint::Mixed(Rc::clone(paint)), adjusted_parts));
        }
        let divisor: F = F::from_u64(total_adjusted_parts).expect("should succeed");
        for i in 0..3 {
            rgb_sum[i] /= divisor;
        }
        let divisor = total_adjusted_parts as f64;
        let mp = MixedPaint::<F> {
            rgb: rgb_sum.into(),
            targeted_rgb: self.targeted_rgb,
            id: self.id.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            finish: finish / divisor,
            transparency: transparency / divisor,
            permanence: permanence / divisor,
            fluorescence: fluorescence / divisor,
            metallicness: metallicness / divisor,
            components,
        };
        Rc::new(mp)
    }
}

#[derive(Debug)]
pub enum Paint<F: ColourComponent> {
    Series(Rc<SeriesPaint<F>>),
    Mixed(Rc<MixedPaint<F>>),
}

impl<F: ColourComponent + ShapeConsts> MakeColouredShape<F> for Paint<F> {
    fn coloured_shape(&self) -> ColouredShape<F> {
        match self {
            Paint::Series(paint) => paint.coloured_shape(),
            Paint::Mixed(paint) => paint.coloured_shape(),
        }
    }
}

impl<F: ColourComponent> ColourInterface<F> for Paint<F> {
    fn rgb(&self) -> RGB<F> {
        match self {
            Paint::Series(paint) => paint.rgb(),
            Paint::Mixed(paint) => paint.rgb(),
        }
    }

    fn rgba(&self, alpha: F) -> [F; 4] {
        match self {
            Paint::Series(paint) => paint.rgba(alpha),
            Paint::Mixed(paint) => paint.rgba(alpha),
        }
    }

    fn hue(&self) -> Option<Hue<F>> {
        match self {
            Paint::Series(paint) => paint.hue(),
            Paint::Mixed(paint) => paint.hue(),
        }
    }

    fn hue_angle(&self) -> Option<Degrees<F>> {
        match self {
            Paint::Series(paint) => paint.hue_angle(),
            Paint::Mixed(paint) => paint.hue_angle(),
        }
    }

    fn is_grey(&self) -> bool {
        match self {
            Paint::Series(paint) => paint.is_grey(),
            Paint::Mixed(paint) => paint.is_grey(),
        }
    }

    fn chroma(&self) -> F {
        match self {
            Paint::Series(paint) => paint.chroma(),
            Paint::Mixed(paint) => paint.chroma(),
        }
    }

    fn max_chroma_rgb(&self) -> RGB<F> {
        match self {
            Paint::Series(paint) => paint.max_chroma_rgb(),
            Paint::Mixed(paint) => paint.max_chroma_rgb(),
        }
    }

    fn greyness(&self) -> F {
        match self {
            Paint::Series(paint) => paint.greyness(),
            Paint::Mixed(paint) => paint.greyness(),
        }
    }

    fn value(&self) -> F {
        match self {
            Paint::Series(paint) => paint.value(),
            Paint::Mixed(paint) => paint.value(),
        }
    }

    fn monochrome_rgb(&self) -> RGB<F> {
        match self {
            Paint::Series(paint) => paint.monochrome_rgb(),
            Paint::Mixed(paint) => paint.monochrome_rgb(),
        }
    }

    fn warmth(&self) -> F {
        match self {
            Paint::Series(paint) => paint.warmth(),
            Paint::Mixed(paint) => paint.warmth(),
        }
    }

    fn warmth_rgb(&self) -> RGB<F> {
        match self {
            Paint::Series(paint) => paint.warmth_rgb(),
            Paint::Mixed(paint) => paint.warmth_rgb(),
        }
    }

    fn best_foreground_rgb(&self) -> RGB<F> {
        match self {
            Paint::Series(paint) => paint.best_foreground_rgb(),
            Paint::Mixed(paint) => paint.best_foreground_rgb(),
        }
    }
}

impl<F: ColourComponent> BasicPaintIfce<F> for Paint<F> {
    fn id(&self) -> &str {
        match self {
            Paint::Series(paint) => paint.id(),
            Paint::Mixed(paint) => paint.id(),
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            Paint::Series(paint) => paint.name(),
            Paint::Mixed(paint) => paint.name(),
        }
    }

    fn notes(&self) -> Option<&str> {
        match self {
            Paint::Series(paint) => paint.notes(),
            Paint::Mixed(paint) => paint.notes(),
        }
    }

    fn finish(&self) -> Finish {
        match self {
            Paint::Series(paint) => paint.finish(),
            Paint::Mixed(paint) => paint.finish(),
        }
    }

    fn transparency(&self) -> Transparency {
        match self {
            Paint::Series(paint) => paint.transparency(),
            Paint::Mixed(paint) => paint.transparency(),
        }
    }

    fn fluorescence(&self) -> Fluorescence {
        match self {
            Paint::Series(paint) => paint.fluorescence(),
            Paint::Mixed(paint) => paint.fluorescence(),
        }
    }

    fn permanence(&self) -> Permanence {
        match self {
            Paint::Series(paint) => paint.permanence(),
            Paint::Mixed(paint) => paint.permanence(),
        }
    }

    fn metallicness(&self) -> Metallicness {
        match self {
            Paint::Series(paint) => paint.metallicness(),
            Paint::Mixed(paint) => paint.metallicness(),
        }
    }
}
