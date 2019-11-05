// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub mod attributes;
pub mod characteristics;
pub mod colour_mix;
pub mod drawing;

pub use colour_math::{ColourComponent, ColourInterface};

pub trait ColouredItem<F, CI>
where
    F: ColourComponent,
    CI: ColourInterface<F>,
{
    fn colour(&self) -> &CI;
}

#[cfg(test)]
mod tests {
    use super::*;
    use colour_math::{Colour, RGB};

    struct ColourWrapper {
        colour: Colour<f64>,
    }

    impl ColouredItem<f64, Colour<f64>> for ColourWrapper {
        fn colour(&self) -> &Colour<f64> {
            &self.colour
        }
    }

    struct RGBWrapper {
        rgb: RGB<f64>,
    }

    impl ColouredItem<f64, RGB<f64>> for RGBWrapper {
        fn colour(&self) -> &RGB<f64> {
            &self.rgb
        }
    }

    #[test]
    fn rgb_wrapper() {
        let wrapper = RGBWrapper { rgb: RGB::YELLOW };
        assert_eq!(wrapper.colour().chroma(), 1.0);
        assert_eq!(wrapper.colour().rgb(), RGB::YELLOW);
    }

    #[test]
    fn colour_wrapper() {
        let wrapper = ColourWrapper {
            colour: Colour::<f64>::from(RGB::CYAN),
        };
        assert_eq!(wrapper.colour().chroma(), 1.0);
        assert_eq!(wrapper.colour().rgb(), RGB::CYAN);
    }
}
