// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use num::Integer;

use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

use apaint_boilerplate::{BasicPaint, Colour};

use crate::{
    characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency},
    series::SeriesPaint,
};

#[derive(Debug, Colour, BasicPaint)]
pub struct MixedPaint<F: ColourComponent> {
    rgb: RGB<F>,
    id: String,
    name: String,
    notes: String,
    finish: Finish,
    transparency: Transparency,
    permanence: Permanence,
    fluorescence: Fluorescence,
    metallicness: Metallicness,
    components: Vec<(Paint<F>, u64)>,
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
}

impl<F: ColourComponent> MixedPaintBuilder<F> {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: String::new(),
            notes: String::new(),
            series_components: vec![],
            mixture_components: vec![],
        }
    }

    pub fn build(self) -> Rc<MixedPaint<F>> {
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
        let mut total_adjustec_parts: u64 = 0;
        let mut rgb_sum: [F; 3] = [F::ZERO, F::ZERO, F::ZERO];
        for (paint, parts) in self.series_components.iter() {
            let adjusted_parts = parts / gcd;
            total_adjustec_parts += adjusted_parts;
            let rgb = paint.rgb();
            for i in 0..3 {
                rgb_sum[i] +=
                    rgb[i as u8] * F::from_u64(adjusted_parts).expect("no problems expected");
            }
            components.push((Paint::Series(Rc::clone(paint)), adjusted_parts));
        }
        for (paint, parts) in self.mixture_components.iter() {
            let adjusted_parts = parts / gcd;
            total_adjustec_parts += adjusted_parts;
            let rgb = paint.rgb();
            for i in 0..3 {
                rgb_sum[i] +=
                    rgb[i as u8] * F::from_u64(adjusted_parts).expect("no problems expected");
            }
            components.push((Paint::Mixed(Rc::clone(paint)), adjusted_parts));
        }
        let divisor: F = F::from_u64(total_adjustec_parts).expect("should succeed");
        for i in 0..3 {
            rgb_sum[i] /= divisor;
        }
        let mp = MixedPaint::<F> {
            rgb: rgb_sum.into(),
            id: self.id,
            name: self.name,
            notes: self.notes,
            finish: Finish::default(),
            transparency: Transparency::default(),
            permanence: Permanence::default(),
            fluorescence: Fluorescence::default(),
            metallicness: Metallicness::default(),
            components,
        };
        Rc::new(mp)
    }
}
