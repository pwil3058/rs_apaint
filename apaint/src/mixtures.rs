// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    io::{Read, Write},
    rc::Rc,
};

use crypto_hash::{Algorithm, Hasher};
use gcd::Gcd;

use colour_math::{
    beigui::hue_wheel::{ColouredShape, MakeColouredShape, Shape},
    mixing::SubtractiveMixer,
    Angle, Chroma, ColourBasics, Hue, LightLevel, Prop, HCV, RGB,
};

use colour_math_derive::Colour;

use crate::characteristics::CharacteristicMixer;
use crate::{
    characteristics::{
        Finish, Fluorescence, FuzzyCharacteristic, Metallicness, Permanence, Transparency,
    },
    series::{SeriesId, SeriesPaint, SeriesPaintFinder},
    BasicPaintIfce, ColourAttributes, Greyness, LabelText, TooltipText, Value, Warmth,
};

// TODO: make an untargeted version of TargetedMixture
#[derive(Debug, Colour)]
pub struct Mixture {
    colour: HCV,
    #[cfg(feature = "targeted_mixtures")]
    targeted_colour: Option<HCV>,
    id: String,
    name: String,
    notes: String,
    finish: FuzzyCharacteristic<Finish>,
    transparency: FuzzyCharacteristic<Transparency>,
    permanence: FuzzyCharacteristic<Permanence>,
    fluorescence: FuzzyCharacteristic<Fluorescence>,
    metallicness: FuzzyCharacteristic<Metallicness>,
    components: Vec<(Paint, u64)>,
}

impl Mixture {
    #[cfg(feature = "targeted_mixtures")]
    pub fn targeted_rgb<L: LightLevel>(&self) -> Option<RGB<L>> {
        if let Some(ref colour) = self.targeted_colour {
            Some(colour.rgb::<L>())
        } else {
            None
        }
    }

    #[cfg(feature = "targeted_mixtures")]
    pub fn targeted_colour(&self) -> Option<HCV> {
        if let Some(colour) = self.targeted_colour {
            Some(colour)
        } else {
            None
        }
    }

    #[cfg(feature = "targeted_mixtures")]
    pub fn targeted_rgb_shape(&self) -> ColouredShape {
        let tooltip_text = format!("Target for: {}", self.tooltip_text());
        let id = self.targeted_rgb_id();
        ColouredShape::new(
            &self.targeted_colour.expect("programmer error"),
            &id,
            &tooltip_text,
            Shape::Circle,
        )
    }

    #[cfg(feature = "targeted_mixtures")]
    pub fn targeted_rgb_id(&self) -> String {
        format!("TARGET({})", self.id)
    }

    pub fn components(&self) -> impl Iterator<Item = &(Paint, u64)> {
        self.components.iter()
    }
}

impl BasicPaintIfce for Mixture {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> Option<&str> {
        if self.name.is_empty() {
            None
        } else {
            Some(&self.name)
        }
    }

    fn notes(&self) -> Option<&str> {
        if self.notes.is_empty() {
            None
        } else {
            Some(&self.notes)
        }
    }

    fn finish(&self) -> Finish {
        self.finish.characteristic()
    }

    fn transparency(&self) -> Transparency {
        self.transparency.characteristic()
    }

    fn fluorescence(&self) -> Fluorescence {
        self.fluorescence.characteristic()
    }

    fn permanence(&self) -> Permanence {
        self.permanence.characteristic()
    }

    fn metallicness(&self) -> Metallicness {
        self.metallicness.characteristic()
    }
}

impl TooltipText for Mixture {
    fn tooltip_text(&self) -> String {
        let mut string = self.label_text();
        if let Some(notes) = self.notes() {
            string.push('\n');
            string.push_str(notes);
        };

        string
    }
}

impl LabelText for Mixture {
    fn label_text(&self) -> String {
        if let Some(name) = self.name() {
            format!("Mix {}: {}", self.id, name)
        } else if let Some(notes) = self.notes() {
            format!("Mix {}: {}", self.id, notes)
        } else {
            format!("Mix {}: {}", self.id, self.rgb::<u8>().pango_string())
        }
    }
}

impl MakeColouredShape for Mixture {
    fn coloured_shape(&self) -> ColouredShape {
        let tooltip_text = self.tooltip_text();
        ColouredShape::new(&self.colour, &self.id, &tooltip_text, Shape::Diamond)
    }
}

impl PartialEq for Mixture {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Mixture {}

impl PartialOrd for Mixture {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.id.cmp(&other.id) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => Some(Ordering::Equal),
        }
    }
}

impl Ord for Mixture {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug)]
pub struct MixingSession {
    notes: String,
    mixtures: Vec<Rc<Mixture>>,
}

impl Default for MixingSession {
    fn default() -> Self {
        Self {
            notes: String::new(),
            mixtures: vec![],
        }
    }
}

impl MixingSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }

    pub fn set_notes(&mut self, notes: &str) {
        self.notes = notes.to_string()
    }

    pub fn mixtures(&self) -> impl Iterator<Item = &Rc<Mixture>> {
        self.mixtures.iter()
    }

    pub fn series_paints(&self) -> Vec<Rc<SeriesPaint>> {
        let mut v = vec![];

        for mixture in self.mixtures.iter() {
            for (paint, _parts) in mixture.components.iter() {
                if let Paint::Series(series_paint) = paint {
                    match v.binary_search_by_key(&series_paint.id(), |p: &Rc<SeriesPaint>| p.id()) {
                        Ok(_) => (),
                        Err(index) => v.insert(index, Rc::clone(series_paint)),
                    }
                }
            }
        }

        v
    }

    pub fn add_mixture(&mut self, mixture: &Rc<Mixture>) -> Option<Rc<Mixture>> {
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

    pub fn mixture(&self, id: &str) -> Option<&Rc<Mixture>> {
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

impl MixingSession {
    pub fn read<R: Read>(
        reader: &mut R,
        series_paint_finder: &Rc<impl SeriesPaintFinder>,
    ) -> Result<Self, crate::Error> {
        let saved_session = SaveableMixingSession::read(reader)?;
        let mixing_session = saved_session.mixing_session(series_paint_finder)?;
        Ok(mixing_session)
    }
}

impl MixingSession {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<Vec<u8>, crate::Error> {
        SaveableMixingSession::from(self).write(writer)
    }

    pub fn digest(&self) -> Result<Vec<u8>, crate::Error> {
        SaveableMixingSession::from(self).digest()
    }
}

#[derive(Debug)]
pub struct MixtureBuilder {
    id: String,
    name: String,
    notes: String,
    series_components: Vec<(Rc<SeriesPaint>, u64)>,
    mixture_components: Vec<(Rc<Mixture>, u64)>,
    #[cfg(feature = "targeted_mixtures")]
    targeted_colour: Option<HCV>,
}

impl MixtureBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: String::new(),
            notes: String::new(),
            series_components: vec![],
            mixture_components: vec![],
            #[cfg(feature = "targeted_mixtures")]
            targeted_colour: None,
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

    #[cfg(feature = "targeted_mixtures")]
    pub fn targeted_colour(&mut self, colour: &impl ColourBasics) -> &mut Self {
        self.targeted_colour = Some(colour.hcv());
        self
    }

    pub fn series_paint_components(
        &mut self,
        components: Vec<(Rc<SeriesPaint>, u64)>,
    ) -> &mut Self {
        self.series_components = components;
        self
    }

    pub fn series_paint_component(&mut self, component: (Rc<SeriesPaint>, u64)) -> &mut Self {
        self.series_components.push(component);
        self
    }

    pub fn mixed_paint_components(&mut self, components: Vec<(Rc<Mixture>, u64)>) -> &mut Self {
        self.mixture_components = components;
        self
    }

    pub fn mixed_paint_component(&mut self, component: (Rc<Mixture>, u64)) -> &mut Self {
        self.mixture_components.push(component);
        self
    }

    pub fn build(&self) -> Rc<Mixture> {
        debug_assert!((self.series_components.len() + self.mixture_components.len()) > 0);
        let mut gcd: u64 = 0;
        for (_, parts) in self.series_components.iter() {
            debug_assert!(*parts > 0);
            gcd = gcd.gcd(*parts as u64);
        }
        for (_, parts) in self.mixture_components.iter() {
            debug_assert!(*parts > 0);
            gcd = gcd.gcd(*parts as u64);
        }
        debug_assert!(gcd > 0);
        let mut components = vec![];
        let mut finish_mix = CharacteristicMixer::<Finish>::new();
        let mut transparency_mix = CharacteristicMixer::<Transparency>::new();
        let mut permanence_mix = CharacteristicMixer::<Permanence>::new();
        let mut fluorescence_mix = CharacteristicMixer::<Fluorescence>::new();
        let mut metallicness_mix = CharacteristicMixer::<Metallicness>::new();
        let mut colour_mix = SubtractiveMixer::new();
        for (paint, parts) in self.series_components.iter() {
            let adjusted_parts = *parts / gcd;
            colour_mix.add(&paint.hcv(), adjusted_parts);
            finish_mix.add(paint.finish(), adjusted_parts);
            transparency_mix.add(paint.transparency(), adjusted_parts);
            permanence_mix.add(paint.permanence(), adjusted_parts);
            fluorescence_mix.add(paint.fluorescence(), adjusted_parts);
            metallicness_mix.add(paint.metallicness(), adjusted_parts);
            components.push((Paint::Series(Rc::clone(paint)), adjusted_parts as u64));
        }
        for (paint, parts) in self.mixture_components.iter() {
            let adjusted_parts = *parts / gcd;
            colour_mix.add(&paint.hcv(), adjusted_parts);
            finish_mix.add_value(paint.finish, adjusted_parts);
            transparency_mix.add_value(paint.transparency, adjusted_parts);
            permanence_mix.add_value(paint.permanence, adjusted_parts);
            fluorescence_mix.add_value(paint.fluorescence, adjusted_parts);
            metallicness_mix.add_value(paint.metallicness, adjusted_parts);
            components.push((Paint::Mixed(Rc::clone(paint)), adjusted_parts as u64));
        }
        let mp = Mixture {
            colour: colour_mix.mixed_colour().unwrap(),
            #[cfg(feature = "targeted_mixtures")]
            targeted_colour: self.targeted_colour,
            id: self.id.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            finish: finish_mix.characteristic_value().expect("programmer error"),
            transparency: transparency_mix
                .characteristic_value()
                .expect("programmer error"),
            permanence: permanence_mix
                .characteristic_value()
                .expect("programmer error"),
            fluorescence: fluorescence_mix
                .characteristic_value()
                .expect("programmer error"),
            metallicness: metallicness_mix
                .characteristic_value()
                .expect("programmer error"),
            components,
        };
        Rc::new(mp)
    }
}

#[derive(Debug, PartialEq)]
pub enum Paint {
    Series(Rc<SeriesPaint>),
    Mixed(Rc<Mixture>),
}

impl MakeColouredShape for Paint {
    fn coloured_shape(&self) -> ColouredShape {
        match self {
            Paint::Series(paint) => paint.coloured_shape(),
            Paint::Mixed(paint) => paint.coloured_shape(),
        }
    }
}

impl ColourBasics for Paint {
    fn hue(&self) -> Option<Hue> {
        match self {
            Paint::Series(paint) => paint.hue(),
            Paint::Mixed(paint) => paint.hue(),
        }
    }

    fn hue_angle(&self) -> Option<Angle> {
        match self {
            Paint::Series(paint) => paint.hue_angle(),
            Paint::Mixed(paint) => paint.hue_angle(),
        }
    }

    fn hue_rgb<L: LightLevel>(&self) -> Option<RGB<L>> {
        match self {
            Paint::Series(paint) => paint.hue_rgb::<L>(),
            Paint::Mixed(paint) => paint.hue_rgb::<L>(),
        }
    }

    fn hue_hcv(&self) -> Option<HCV> {
        match self {
            Paint::Series(paint) => paint.hue_hcv(),
            Paint::Mixed(paint) => paint.hue_hcv(),
        }
    }

    fn is_grey(&self) -> bool {
        match self {
            Paint::Series(paint) => paint.is_grey(),
            Paint::Mixed(paint) => paint.is_grey(),
        }
    }

    fn chroma(&self) -> Chroma {
        match self {
            Paint::Series(paint) => paint.chroma(),
            Paint::Mixed(paint) => paint.chroma(),
        }
    }

    fn chroma_prop(&self) -> Prop {
        match self {
            Paint::Series(paint) => paint.chroma_prop(),
            Paint::Mixed(paint) => paint.chroma_prop(),
        }
    }

    fn value(&self) -> Value {
        match self {
            Paint::Series(paint) => paint.value(),
            Paint::Mixed(paint) => paint.value(),
        }
    }

    fn greyness(&self) -> Greyness {
        match self {
            Paint::Series(paint) => paint.greyness(),
            Paint::Mixed(paint) => paint.greyness(),
        }
    }

    fn warmth(&self) -> Warmth {
        match self {
            Paint::Series(paint) => paint.warmth(),
            Paint::Mixed(paint) => paint.warmth(),
        }
    }

    fn hcv(&self) -> HCV {
        match self {
            Paint::Series(paint) => paint.hcv(),
            Paint::Mixed(paint) => paint.hcv(),
        }
    }

    fn rgb<L: LightLevel>(&self) -> RGB<L> {
        match self {
            Paint::Series(paint) => paint.rgb(),
            Paint::Mixed(paint) => paint.rgb(),
        }
    }

    fn monochrome_hcv(&self) -> HCV {
        match self {
            Paint::Series(paint) => paint.monochrome_hcv(),
            Paint::Mixed(paint) => paint.monochrome_hcv(),
        }
    }

    fn monochrome_rgb<L: LightLevel>(&self) -> RGB<L> {
        match self {
            Paint::Series(paint) => paint.monochrome_rgb::<L>(),
            Paint::Mixed(paint) => paint.monochrome_rgb::<L>(),
        }
    }

    fn best_foreground(&self) -> HCV {
        match self {
            Paint::Series(paint) => paint.best_foreground(),
            Paint::Mixed(paint) => paint.best_foreground(),
        }
    }
}

impl ColourAttributes for Paint {}

impl BasicPaintIfce for Paint {
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

impl From<&Rc<SeriesPaint>> for SaveablePaint {
    fn from(paint: &Rc<SeriesPaint>) -> Self {
        SaveablePaint::Series(paint.series_id().into(), paint.id().to_string())
    }
}

impl From<&Rc<Mixture>> for SaveablePaint {
    fn from(paint: &Rc<Mixture>) -> Self {
        SaveablePaint::Mixed(paint.id().to_string())
    }
}

impl From<&Paint> for SaveablePaint {
    fn from(paint: &Paint) -> Self {
        match paint {
            Paint::Series(paint) => paint.into(),
            Paint::Mixed(paint) => paint.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveableMixture {
    #[cfg(feature = "targeted_mixtures")]
    targeted_colour: Option<HCV>,
    id: String,
    name: String,
    notes: String,
    components: Vec<(SaveablePaint, u64)>,
}

impl From<&Rc<Mixture>> for SaveableMixture {
    fn from(rcmp: &Rc<Mixture>) -> Self {
        let components = rcmp
            .components
            .iter()
            .map(|(paint, parts)| (SaveablePaint::from(paint), *parts))
            .collect();
        Self {
            #[cfg(feature = "targeted_mixtures")]
            targeted_colour: rcmp.targeted_colour,
            id: rcmp.id.to_string(),
            name: rcmp.name.to_string(),
            notes: rcmp.notes.to_string(),
            components,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveableMixingSession {
    notes: String,
    mixtures: Vec<SaveableMixture>,
}

impl From<&MixingSession> for SaveableMixingSession {
    fn from(session: &MixingSession) -> Self {
        let mixtures = session.mixtures.iter().map(SaveableMixture::from).collect();
        Self {
            notes: session.notes.to_string(),
            mixtures,
        }
    }
}

impl SaveableMixingSession {
    pub fn mixing_session(
        &self,
        series_paint_finder: &Rc<impl SeriesPaintFinder>,
    ) -> Result<MixingSession, crate::Error> {
        let mut mixtures: Vec<Rc<Mixture>> = vec![];
        for saved_mixture in self.mixtures.iter() {
            let mut mixture_builder = MixtureBuilder::new(&saved_mixture.id);
            mixture_builder.name(&saved_mixture.name);
            mixture_builder.notes(&saved_mixture.notes);
            #[cfg(feature = "targeted_mixtures")]
            if let Some(targeted_colour) = saved_mixture.targeted_colour {
                mixture_builder.targeted_colour(&targeted_colour);
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

impl<'de> SaveableMixingSession {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, crate::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        let session: Self = serde_json::from_str(&string)?;
        Ok(session)
    }
}

impl SaveableMixingSession {
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
    use crate::series::{BasicPaintSpec, SeriesPaintSeries, SeriesPaintSeriesSpec};
    use colour_math::{HueConstants, HCV, RGB};

    #[test]
    fn save_and_recover() {
        let mut series_spec = SeriesPaintSeriesSpec::default();
        series_spec.set_proprietor("owner");
        series_spec.set_series_name("series name");
        assert!(series_spec.paints().next().is_none());
        series_spec.add(&BasicPaintSpec::new(&RGB::<f64>::RED, "red"));
        series_spec.add(&BasicPaintSpec::new(&HCV::YELLOW, "yellow"));
        let series = Rc::new(SeriesPaintSeries::from(&series_spec));
        let mut session = MixingSession::new();
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
        let read_session = MixingSession::read(&mut &buffer[..], &series).unwrap();
        assert_eq!(digest, read_session.digest().unwrap());
        assert_eq!(session.notes(), read_session.notes());
        assert_eq!(session.mixtures.len(), read_session.mixtures.len());
        for (mix1, mix2) in session.mixtures().zip(read_session.mixtures()) {
            assert_eq!(mix1, mix2);
        }
    }
}
