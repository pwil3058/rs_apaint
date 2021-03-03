// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub use colour_math::*;

#[derive(Debug)]
pub struct ColourMixer<F: LightLevel> {
    pub rgb_sum: [F; 3],
    pub total_parts: u64,
}

impl<F: LightLevel> Default for ColourMixer<F> {
    fn default() -> Self {
        Self {
            rgb_sum: [F::ZERO, F::ZERO, F::ZERO],
            total_parts: 0,
        }
    }
}

impl<F> ColourMixer<F>
where
    F: LightLevel
        + num_traits::cast::FromPrimitive
        + std::ops::Div<Output = F>
        + std::ops::Mul<Output = F>
        + std::ops::AddAssign,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.rgb_sum = [F::ZERO, F::ZERO, F::ZERO];
        self.total_parts = 0;
    }

    pub fn mixture(&self) -> Option<RGB<F>> {
        if self.total_parts > 0 {
            let divisor = F::from_u64(self.total_parts).expect("should be valid");
            let array: [F; 3] = [
                self.rgb_sum[0] / divisor,
                self.rgb_sum[1] / divisor,
                self.rgb_sum[2] / divisor,
            ];
            Some(array.into())
        } else {
            None
        }
    }

    pub fn add(&mut self, rgb: &RGB<F>, parts: u64) {
        self.total_parts += parts;
        let multiplier = F::from_u64(parts).expect("should be valid");
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
        let mut colour_mixer = ColourMixer::<f64>::new();
        assert_eq!(colour_mixer.mixture(), None);
        colour_mixer.add(&RGB::RED, 10);
        assert_eq!(colour_mixer.mixture(), Some(RGB::RED));
        colour_mixer.add(&RGB::BLUE, 10);
        colour_mixer.add(&RGB::GREEN, 10);
        assert!(colour_mixer.mixture().unwrap().is_grey());
        assert_eq!(
            f64::from(colour_mixer.mixture().unwrap().value()),
            1.0_f64 / 3.0
        );
    }
}
