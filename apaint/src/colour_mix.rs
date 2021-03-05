// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub use colour_math::*;

#[derive(Debug, Default)]
pub struct ColourMixer {
    pub rgb_sum: [f64; 3],
    pub total_parts: u64,
}

impl ColourMixer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.rgb_sum = [0.0, 0.0, 0.0];
        self.total_parts = 0;
    }

    pub fn mixture(&self) -> Option<HCV> {
        if self.total_parts > 0 {
            let divisor = self.total_parts as f64;
            let array: [Prop; 3] = [
                (self.rgb_sum[0] / divisor).into(),
                (self.rgb_sum[1] / divisor).into(),
                (self.rgb_sum[2] / divisor).into(),
            ];
            Some(HCV::from(array))
        } else {
            None
        }
    }

    pub fn mixture_rgb<L: LightLevel>(&self) -> Option<RGB<L>> {
        if self.total_parts > 0 {
            let divisor = self.total_parts as f64;
            let array: [Prop; 3] = [
                (self.rgb_sum[0] / divisor).into(),
                (self.rgb_sum[1] / divisor).into(),
                (self.rgb_sum[2] / divisor).into(),
            ];
            Some(RGB::<L>::from(array))
        } else {
            None
        }
    }

    pub fn add(&mut self, colour: &impl ColourBasics, parts: u64) {
        self.total_parts += parts;
        let multiplier = parts as f64;
        let rgb = colour.rgb::<f64>();
        self.rgb_sum[0] += rgb[CCI::Red] * multiplier;
        self.rgb_sum[1] += rgb[CCI::Green] * multiplier;
        self.rgb_sum[2] += rgb[CCI::Blue] * multiplier;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_colour_mix_test() {
        let mut colour_mixer = ColourMixer::new();
        assert_eq!(colour_mixer.mixture(), None);
        colour_mixer.add(&RGB::<f64>::RED, 10);
        assert_eq!(colour_mixer.mixture(), Some(HCV::RED));
        colour_mixer.add(&HCV::BLUE, 10);
        colour_mixer.add(&RGB::<u16>::GREEN, 10);
        assert!(colour_mixer.mixture().unwrap().is_grey());
        assert_eq!(
            f64::from(colour_mixer.mixture().unwrap().value()),
            1.0_f64 / 3.0
        );
    }
}
