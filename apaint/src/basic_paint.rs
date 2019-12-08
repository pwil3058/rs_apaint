// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use apaint_boilerplate::Colour;
use colour_math::{ColourComponent, ColourInterface, ScalarAttribute, RGB};

use crate::characteristics::{Finish, Fluorescence, Metallicness, Permanence, Transparency};
use crate::hue_wheel::{ColouredShape, MakeColouredShape, Shape, ShapeConsts};
use crate::{BasicPaintIfce, BasicPaintSpec, FromSpec};

#[derive(Debug, Deserialize, Serialize, Colour, Clone)]
pub struct BasicPaint<F: ColourComponent> {
    rgb: RGB<F>,
    id: String,
    name: String,
    notes: String,
    finish: Finish,
    transparency: Transparency,
    permanence: Permanence,
    fluorescence: Fluorescence,
    metallicness: Metallicness,
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

    fn finish(&self) -> Finish {
        self.finish
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn fluorescence(&self) -> Fluorescence {
        self.fluorescence
    }

    fn permanence(&self) -> Permanence {
        self.permanence
    }

    fn metallicness(&self) -> Metallicness {
        self.metallicness
    }
}

impl<F: ColourComponent> FromSpec<F> for BasicPaint<F> {
    fn from_spec(spec: &BasicPaintSpec<F>) -> Self {
        Self {
            rgb: spec.rgb,
            id: spec.id.to_string(),
            name: spec.name.to_string(),
            notes: spec.notes.to_string(),
            finish: spec.finish,
            transparency: spec.transparency,
            permanence: spec.permanence,
            fluorescence: spec.fluorescence,
            metallicness: spec.metallicness,
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
