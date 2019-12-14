// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use num::Integer;

use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

use apaint_boilerplate::{BasicPaint, Colour};

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    series::SeriesPaint,
    BasicPaintIfce,
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

impl<F: ColourComponent> MixedPaint<F> {
    pub fn targeted_rgb(&self) -> Option<&RGB<F>> {
        if let Some(ref rgb) = self.targeted_rgb {
            Some(rgb)
        } else {
            None
        }
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

#[derive(Debug)]
pub enum Paint<F: ColourComponent> {
    Series(Rc<SeriesPaint<F>>),
    Mixed(Rc<MixedPaint<F>>),
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

    pub fn build(self) -> Rc<MixedPaint<F>> {
        debug_assert!((self.series_components.len() + self.mixture_components.len()) > 0);
        let mut gcd: u64 = 1;
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
            id: self.id,
            name: self.name,
            notes: self.notes,
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
