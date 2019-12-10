// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{fs::File, rc::Rc};

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::recollections;
use pw_gix::wrapper::*;

use apaint::{characteristics::CharacteristicType, LabelText, TooltipText};

use apaint_boilerplate::Colour;

use apaint::spec::BasicPaintSeriesSpec;
use apaint_gtk::{
    colour::*,
    factory::BasicPaintFactory,
    mixer::{
        component::{PartsSpinButtonBox, RcPartsSpinButtonBox},
        targeted::TargetedPaintEntry,
    },
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
    let mp_entry = TargetedPaintEntry::new(&[
        ScalarAttribute::Value,
        ScalarAttribute::Greyness,
        ScalarAttribute::Chroma,
    ]);
    vbox.pack_start(&mp_entry.pwo(), false, false, 0);
    let spinners = PartsSpinButtonBox::new("Paints", 4, true);
    let dummy = Dummy { rgb: RGB::CYAN };
    spinners.add_paint(&dummy);
    let dummy = Dummy { rgb: RGB::YELLOW };
    spinners.add_paint(&dummy);
    for rgb in RGB::PRIMARIES.iter() {
        spinners.add_paint(&Dummy { rgb: *rgb });
    }
    for rgb in RGB::SECONDARIES.iter() {
        spinners.add_paint(&Dummy { rgb: *rgb });
    }
    for rgb in RGB::GREYS.iter() {
        spinners.add_paint(&Dummy { rgb: *rgb });
    }
    vbox.pack_start(&spinners.pwo(), false, false, 0);
    let spinners_c = Rc::clone(&spinners);
    spinners.connect_contributions_changed(move || {
        println!("changed: {:?}", spinners_c.rgb_contributions())
    });
    let spinners_c = Rc::clone(&spinners);
    spinners.connect_removal_requested(move |paint| spinners_c.remove_paint(paint));
    let binder = SeriesBinder::new(
        &[("test", "Test", None, "testing", 0).into()],
        &[ScalarAttribute::Value, ScalarAttribute::Greyness],
        &[
            CharacteristicType::Finish,
            CharacteristicType::Transparency,
            CharacteristicType::Fluorescence,
            CharacteristicType::Metallicness,
        ],
    );
    binder.connect_popup_menu_item("test", |paint| println!("{:?}", paint));
    let mut file = File::open("./test_saved_file.json").unwrap();
    let paint_series_spec = BasicPaintSeriesSpec::<f64>::read(&mut file).unwrap();
    binder
        .add_series((&paint_series_spec).into())
        .expect("should be OK");
    vbox.pack_start(&binder.pwo(), true, true, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
