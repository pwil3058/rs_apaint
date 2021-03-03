// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use pw_gix::{
    gtk::{self, BoxExt, ContainerExt, WidgetExt},
    recollections,
    wrapper::*,
};

use apaint::{characteristics::CharacteristicType, LabelText, TooltipText};

use colour_math_derive::Colour;

use apaint::series::{BasicPaintSpec, SeriesId, SeriesPaint};
use apaint_gtk::mixer::targeted::TargetedPaintMixerBuilder;
use apaint_gtk::series::display::PaintDisplayBuilder;
use apaint_gtk::{colour::*, factory::BasicPaintFactoryBuilder};

#[derive(Colour, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Dummy {
    colour: HCV,
}

impl TooltipText for Dummy {
    fn tooltip_text(&self) -> String {
        "tooltip text".to_string()
    }
}

impl LabelText for Dummy {
    fn label_text(&self) -> String {
        "dummy paint".to_string()
    }
}

fn main() {
    recollections::init("./.recollections");
    if gtk::init().is_err() {
        println!("Hello, world!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(
        &BasicPaintFactoryBuilder::new()
            .attributes(&[
                ScalarAttribute::Value,
                ScalarAttribute::Greyness,
                //ScalarAttribute::Chroma,
            ])
            .characteristics(&[
                CharacteristicType::Finish,
                CharacteristicType::Transparency,
                CharacteristicType::Fluorescence,
                CharacteristicType::Metallicness,
            ])
            .build()
            .pwo(),
        false,
        false,
        0,
    );
    let mixer = TargetedPaintMixerBuilder::new()
        .attributes(&[
            ScalarAttribute::Value,
            ScalarAttribute::Greyness,
            ScalarAttribute::Chroma,
        ])
        .characteristics(&[
            CharacteristicType::Finish,
            CharacteristicType::Transparency,
            CharacteristicType::Fluorescence,
            CharacteristicType::Metallicness,
        ])
        .build();
    vbox.pack_start(&mixer.pwo(), false, false, 0);
    // TODO: why do paint and target have different values?
    let mut paint_spec = BasicPaintSpec::new(&RGB::<f64>::from([0.1, 0.3, 0.8]), "id");
    paint_spec.name = "name".to_string();
    paint_spec.notes = "notes".to_string();
    let paint = SeriesPaint::from((&paint_spec, &Rc::new(SeriesId::new("Series", "Owner"))));
    let mut builder = PaintDisplayBuilder::new();
    builder
        .attributes(&[
            ScalarAttribute::Value,
            ScalarAttribute::Greyness,
            ScalarAttribute::Chroma,
        ])
        .characteristics(&[
            CharacteristicType::Finish,
            CharacteristicType::Transparency,
            CharacteristicType::Fluorescence,
            CharacteristicType::Metallicness,
        ])
        .target_colour(Some(&RGB::<f64>::from([0.6, 0.1, 0.7])));
    let display = builder.build(&Rc::new(paint));
    vbox.pack_start(&display.pwo(), true, true, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
