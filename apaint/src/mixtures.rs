// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    io::{Read, Write},
    rc::Rc,
};

use serde::{de::DeserializeOwned, Serialize};

use crypto_hash::{Algorithm, Hasher};
use num::Integer;

use colour_math::{ColourComponent, ColourInterface, Degrees, Hue, ScalarAttribute, RGB};

use apaint_boilerplate::Colour;

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    hue_wheel::{ColouredShape, MakeColouredShape, Shape, ShapeConsts},
    series::{SeriesId, SeriesPaint, SeriesPaintFinder},
    BasicPaintIfce, LabelText, TooltipText,
};

#[derive(Debug, Colour)]
pub struct Mixture<F: ColourComponent> {
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

impl<F: ColourComponent + ShapeConsts> Mixture<F> {
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

    pub fn components(&self) -> impl Iterator<Item = &(Paint<F>, u64)> {
        self.components.iter()
    }
}

impl<F: ColourComponent> BasicPaintIfce<F> for Mixture<F> {
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

impl<F: ColourComponent> TooltipText for Mixture<F> {
    fn tooltip_text(&self) -> String {
        let mut string = self.label_text();
        if let Some(notes) = self.notes() {
            string.push('\n');
            string.push_str(notes);
        };

        string
    }
}

impl<F: ColourComponent> LabelText for Mixture<F> {
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

impl<F: ColourComponent + ShapeConsts> MakeColouredShape<F> for Mixture<F> {
    fn coloured_shape(&self) -> ColouredShape<F> {
        let tooltip_text = self.tooltip_text();
        ColouredShape::new(self.rgb, &self.id, &tooltip_text, Shape::Diamond)
    }
}

impl<F: ColourComponent> PartialEq for Mixture<F> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<F: ColourComponent> Eq for Mixture<F> {}

impl<F: ColourComponent> PartialOrd for Mixture<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.id.cmp(&other.id) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => Some(Ordering::Equal),
        }
    }
}

impl<F: ColourComponent> Ord for Mixture<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug)]
pub struct MixingSession<F: ColourComponent> {
    notes: String,
    mixtures: Vec<Rc<Mixture<F>>>,
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

    pub fn mixtures(&self) -> impl Iterator<Item = &Rc<Mixture<F>>> {
        self.mixtures.iter()
    }

    pub fn series_paints(&self) -> Vec<Rc<SeriesPaint<F>>> {
        let mut v = vec![];

        for mixture in self.mixtures.iter() {
            for (paint, _parts) in mixture.components.iter() {
                if let Paint::Series(series_paint) = paint {
                    match v
                        .binary_search_by_key(&series_paint.id(), |p: &Rc<SeriesPaint<F>>| p.id())
                    {
                        Ok(_) => (),
                        Err(index) => v.insert(index, Rc::clone(series_paint)),
                    }
                }
            }
        }

        v
    }

    pub fn add_mixture(&mut self, mixture: &Rc<Mixture<F>>) -> Option<Rc<Mixture<F>>> {
        debug_assert!(self.is_sorted_unique());
        match self
            .mixtures
            .binary_search_by_key(&mixture.id(), |p| p.id())
        {
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

    pub fn mixture(&self, id: &str) -> Option<&Rc<Mixture<F>>> {
        debug_assert!(self.is_sorted_unique());
        match self.mixtures.binary_search_by_key(&id, |p| p.id()) {
            Ok(index) => self.mixtures.get(index),
            Err(_) => None,
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

impl<F: ColourComponent + DeserializeOwned> MixingSession<F> {
    pub fn read<R: Read>(
        reader: &mut R,
        series_paint_finder: &Rc<impl SeriesPaintFinder<F>>,
    ) -> Result<Self, crate::Error> {
        let saved_session = SaveableMixingSession::read(reader)?;
        let mixing_session = saved_session.mixing_session(series_paint_finder)?;
        Ok(mixing_session)
    }
}

impl<F: ColourComponent + Serialize> MixingSession<F> {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<Vec<u8>, crate::Error> {
        SaveableMixingSession::from(self).write(writer)
    }

    pub fn digest(&self) -> Result<Vec<u8>, crate::Error> {
        SaveableMixingSession::from(self).digest()
    }
}

#[derive(Debug)]
pub struct MixtureBuilder<F: ColourComponent> {
    id: String,
    name: String,
    notes: String,
    series_components: Vec<(Rc<SeriesPaint<F>>, u64)>,
    mixture_components: Vec<(Rc<Mixture<F>>, u64)>,
    targeted_rgb: Option<RGB<F>>,
}

impl<F: ColourComponent> MixtureBuilder<F> {
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

    pub fn series_paint_component(&mut self, component: (Rc<SeriesPaint<F>>, u64)) -> &mut Self {
        self.series_components.push(component);
        self
    }

    pub fn mixed_paint_components(&mut self, components: Vec<(Rc<Mixture<F>>, u64)>) -> &mut Self {
        self.mixture_components = components;
        self
    }

    pub fn mixed_paint_component(&mut self, component: (Rc<Mixture<F>>, u64)) -> &mut Self {
        self.mixture_components.push(component);
        self
    }

    pub fn build(&self) -> Rc<Mixture<F>> {
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
        let mp = Mixture::<F> {
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

#[derive(Debug, PartialEq)]
pub enum Paint<F: ColourComponent> {
    Series(Rc<SeriesPaint<F>>),
    Mixed(Rc<Mixture<F>>),
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

#[derive(Serialize, Deserialize, Debug)]
pub enum SaveablePaint {
    Series(SeriesId, String),
    Mixed(String),
}

impl<F: ColourComponent> From<&Rc<SeriesPaint<F>>> for SaveablePaint {
    fn from(paint: &Rc<SeriesPaint<F>>) -> Self {
        SaveablePaint::Series(paint.series_id().into(), paint.id().to_string())
    }
}

impl<F: ColourComponent> From<&Rc<Mixture<F>>> for SaveablePaint {
    fn from(paint: &Rc<Mixture<F>>) -> Self {
        SaveablePaint::Mixed(paint.id().to_string())
    }
}

impl<F: ColourComponent> From<&Paint<F>> for SaveablePaint {
    fn from(paint: &Paint<F>) -> Self {
        match paint {
            Paint::Series(paint) => paint.into(),
            Paint::Mixed(paint) => paint.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveableMixture<F: ColourComponent> {
    targeted_rgb: Option<RGB<F>>,
    id: String,
    name: String,
    notes: String,
    components: Vec<(SaveablePaint, u64)>,
}

impl<F: ColourComponent> From<&Rc<Mixture<F>>> for SaveableMixture<F> {
    fn from(rcmp: &Rc<Mixture<F>>) -> Self {
        let components = rcmp
            .components
            .iter()
            .map(|(paint, parts)| (SaveablePaint::from(paint), *parts))
            .collect();
        Self {
            targeted_rgb: rcmp.targeted_rgb,
            id: rcmp.id.to_string(),
            name: rcmp.name.to_string(),
            notes: rcmp.notes.to_string(),
            components,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveableMixingSession<F: ColourComponent> {
    notes: String,
    mixtures: Vec<SaveableMixture<F>>,
}

impl<F: ColourComponent> From<&MixingSession<F>> for SaveableMixingSession<F> {
    fn from(session: &MixingSession<F>) -> Self {
        let mixtures = session
            .mixtures
            .iter()
            .map(|p| SaveableMixture::from(p))
            .collect();
        Self {
            notes: session.notes.to_string(),
            mixtures,
        }
    }
}

impl<F: ColourComponent> SaveableMixingSession<F> {
    pub fn mixing_session(
        &self,
        series_paint_finder: &Rc<impl SeriesPaintFinder<F>>,
    ) -> Result<MixingSession<F>, crate::Error> {
        let mut mixtures: Vec<Rc<Mixture<F>>> = vec![];
        for saved_mixture in self.mixtures.iter() {
            let mut mixture_builder = MixtureBuilder::new(&saved_mixture.id);
            mixture_builder.name(&saved_mixture.name);
            mixture_builder.notes(&saved_mixture.notes);
            if let Some(targeted_rgb) = saved_mixture.targeted_rgb {
                mixture_builder.targeted_rgb(&targeted_rgb);
            }
            for saved_component in saved_mixture.components.iter() {
                match &saved_component.0 {
                    SaveablePaint::Series(series_id, id) => {
                        let paint = series_paint_finder.get_series_paint(id, Some(series_id))?;
                        mixture_builder.series_paint_component((paint, saved_component.1));
                    }
                    SaveablePaint::Mixed(id) => {
                        match mixtures.binary_search_by_key(&id.as_str(), |p| p.id()) {
                            Ok(index) => {
                                let paint = mixtures.get(index).expect("binary searched index");
                                mixture_builder
                                    .mixed_paint_component((Rc::clone(paint), saved_component.1));
                            }
                            Err(_) => return Err(crate::Error::NotFound(id.to_string())),
                        }
                    }
                }
            }
            let mixture = mixture_builder.build();
            mixtures.push(mixture);
        }
        Ok(MixingSession {
            notes: self.notes.to_string(),
            mixtures,
        })
    }
}

impl<'de, F> SaveableMixingSession<F>
where
    F: ColourComponent + DeserializeOwned,
{
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, crate::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        let session: Self = serde_json::from_str(&string)?;
        Ok(session)
    }
}

impl<F: ColourComponent + Serialize> SaveableMixingSession<F> {
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
    use std::rc::Rc;

    use crate::mixtures::{MixingSession, MixtureBuilder};
    use crate::series::{
        BasicPaintSpec, SeriesId, SeriesPaint, SeriesPaintFinder, SeriesPaintSeries,
        SeriesPaintSeriesSpec,
    };
    use colour_math::RGB;

    impl SeriesPaintFinder<f64> for SeriesPaintSeries<f64> {
        fn get_series_paint(
            &self,
            id: &str,
            _series_id: Option<&SeriesId>,
        ) -> Result<Rc<SeriesPaint<f64>>, crate::Error> {
            if let Some(paint) = self.find(id) {
                Ok(Rc::clone(paint))
            } else {
                Err(crate::Error::NotFound(id.to_string()))
            }
        }
    }

    #[test]
    fn save_and_recover() {
        let mut series_spec = SeriesPaintSeriesSpec::<f64>::default();
        series_spec.set_proprietor("owner");
        series_spec.set_series_name("series name");
        assert!(series_spec.paints().next().is_none());
        series_spec.add(&BasicPaintSpec::new(RGB::<f64>::RED, "red"));
        series_spec.add(&BasicPaintSpec::new(RGB::<f64>::YELLOW, "yellow"));
        let series = Rc::new(SeriesPaintSeries::<f64>::from(&series_spec));
        let mut session = MixingSession::<f64>::new();
        session.set_notes("a test mixing session");
        let red = series.find("red").unwrap();
        let yellow = series.find("red").unwrap();
        let mix = vec![(Rc::clone(red), 1), (Rc::clone(yellow), 1)];
        let mixture = MixtureBuilder::new("#001")
            .series_paint_components(mix)
            .name("orange")
            .build();
        session.add_mixture(&mixture);
        let mut buffer: Vec<u8> = vec![];
        let digest = session.write(&mut buffer).unwrap();
        let read_session = MixingSession::<f64>::read(&mut &buffer[..], &series).unwrap();
        assert_eq!(digest, read_session.digest().unwrap());
        assert_eq!(session.notes(), read_session.notes());
        assert_eq!(session.mixtures.len(), read_session.mixtures.len());
        for (mix1, mix2) in session.mixtures().zip(read_session.mixtures()) {
            assert_eq!(mix1, mix2);
        }
    }
}
