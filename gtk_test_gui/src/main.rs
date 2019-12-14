// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::fs::File;

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::recollections;
use pw_gix::wrapper::*;

use apaint::{characteristics::CharacteristicType, LabelText, TooltipText};

use apaint_boilerplate::Colour;

use apaint::spec::BasicPaintSeriesSpec;
use apaint_gtk::mixer::targeted::TargetedPaintMixer;
use apaint_gtk::{
    colour::*,
    factory::BasicPaintFactory,
    series::{RcSeriesBinder, SeriesBinder},
};

#[derive(Colour, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[component = "f64"]
struct Dummy {
    rgb: RGB,
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
        &BasicPaintFactory::new(
            &[
                ScalarAttribute::Value,
                ScalarAttribute::Greyness,
                //ScalarAttribute::Chroma,
            ],
            &[
                CharacteristicType::Finish,
                CharacteristicType::Transparency,
                CharacteristicType::Fluorescence,
                CharacteristicType::Metallicness,
            ],
        )
        .pwo(),
        false,
        false,
        0,
    );
    let mixer = TargetedPaintMixer::new(
        &[
            ScalarAttribute::Value,
            ScalarAttribute::Greyness,
            ScalarAttribute::Chroma,
        ],
        &[
            CharacteristicType::Finish,
            CharacteristicType::Transparency,
            CharacteristicType::Fluorescence,
            CharacteristicType::Metallicness,
        ],
    );
    vbox.pack_start(&mixer.pwo(), false, false, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
