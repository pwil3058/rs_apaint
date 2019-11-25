// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use apaint_boilerplate::Colour;
use colour_math::{ColourComponent, ColourInterface, Hue, ScalarAttribute, RGB};
use normalised_angles::*;

use crate::hue_wheel::{ColouredShape, MakeColouredShape, Shape, ShapeConsts};
use crate::{BasicPaintIfce, BasicPaintSpec, FromSpec};

#[derive(Debug, Deserialize, Serialize, Colour, Clone)]
pub struct BasicPaint<F: ColourComponent> {
    rgb: RGB<F>,
    id: String,
    name: String,
    notes: String,
}

impl<F: ColourComponent> BasicPaintIfce<F> for BasicPaint<F> {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> Option<&str> {
        if self.name.len() == 0 {
            None
        } else {
            Some(&self.name)
        }
    }

    fn notes(&self) -> Option<&str> {
        if self.notes.len() == 0 {
            None
        } else {
            Some(&self.notes)
        }
    }
}

pub struct BasicPaintBuilder<F: ColourComponent> {
    rgb: RGB<F>,
    id: String,
    name: String,
    notes: String,
}

impl<F: ColourComponent> BasicPaintBuilder<F> {
    pub fn new(id: &str, rgb: &RGB<F>) -> Self {
        Self {
            rgb: *rgb,
            id: id.to_string(),
            name: String::new(),
            notes: String::new(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn notes(mut self, notes: &str) -> Self {
        self.notes = notes.to_string();
        self
    }

    pub fn build(&self) -> BasicPaint<F> {
        BasicPaint {
            rgb: self.rgb,
            id: self.id.to_string(),
            name: self.name.to_string(),
            notes: self.notes.to_string(),
        }
    }
}

impl<F: ColourComponent> FromSpec<F> for BasicPaint<F> {
    fn from_spec(spec: &BasicPaintSpec<F>) -> Self {
        Self {
            rgb: spec.rgb,
            id: spec.id.to_string(),
            name: spec.name.to_string(),
            notes: spec.notes.to_string(),
        }
    }
}

impl<F: ColourComponent + ShapeConsts> MakeColouredShape<F> for BasicPaint<F> {
    fn coloured_shape(&self) -> ColouredShape<F> {
        let tooltip_text = if let Some(name) = self.name() {
            if let Some(notes) = self.notes() {
                format!("{}: {}\n{}", self.id, name, notes)
            } else {
                format!("{}: {}", self.id, name)
            }
        } else if let Some(notes) = self.notes() {
            format!("{}: {}", self.id, notes)
        } else {
            format!("{}: {}", self.id, self.rgb().pango_string())
        };
        ColouredShape::new(self.rgb, &self.id, &tooltip_text, Shape::Square)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::str::FromStr;

    use colour_math::rgb::RGB8;

    use crate::characteristics::*;

    #[test]
    fn builder() {
        let builder =
            BasicPaintBuilder::<f64>::new("71.002", &RGB8::from_str("#F9A7F6").unwrap().into())
                .name("Medium Yellow");
        let paint = builder.build();
        assert_eq!(paint.id(), "71.002");
        assert_eq!(paint.rgb().pango_string(), "#F9A7F6");
        assert_eq!(paint.name(), Some("Medium Yellow"));
        assert_eq!(paint.notes(), None);
        assert_eq!(paint.finish(), Finish::default());
        assert_eq!(paint.permanence(), Permanence::default());
        assert_eq!(paint.transparency(), Transparency::default());
        assert_eq!(paint.fluorescence(), Fluorescence::default());
        assert_eq!(paint.metallicness(), Metallicness::default());
    }
}
