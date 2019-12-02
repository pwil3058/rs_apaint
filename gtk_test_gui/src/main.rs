// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::recollections;
use pw_gix::wrapper::*;

use apaint::basic_paint::BasicPaint;

use apaint::characteristics::CharacteristicType;
use apaint_gtk::{
    colour::{ScalarAttribute, RGB},
    factory::BasicPaintFactory,
    hue_wheel::GtkHueWheel,
    SAV_HAS_CHOSEN_ITEM,
};

fn main() {
    recollections::init("./.recollections");
    if gtk::init().is_err() {
        println!("Hello, world!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(
        &BasicPaintFactory::<BasicPaint<f64>>::new(
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
    let graticule = GtkHueWheel::new(
        &[(
            "add",
            "Add",
            None,
            "Add the selected colour to the colour mixer",
            SAV_HAS_CHOSEN_ITEM,
        )],
        &[
            ScalarAttribute::Value,
            ScalarAttribute::Chroma,
            ScalarAttribute::Warmth,
        ],
    );
    for rgb in RGB::PRIMARIES.iter() {
        graticule.add_item(rgb.into());
    }
    for rgb in RGB::SECONDARIES.iter() {
        graticule.add_item(rgb.into());
    }
    for rgb in RGB::GREYS.iter() {
        graticule.add_item(rgb.into());
    }
    vbox.pack_start(&graticule.pwo(), true, true, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
