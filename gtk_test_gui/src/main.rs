// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::wrapper::*;

use apaint::characteristics::CharacteristicIfce;

use apaint_gtk::characteristics::FinishEntry;
use apaint_gtk::colour::{IdRGB, ScalarAttribute, RGB};
use apaint_gtk::colour_edit::ColourEditor;
use apaint_gtk::factory::BasicPaintEditor;
use apaint_gtk::graticule::GtkGraticule;
use apaint_gtk::list::RGBList;

fn main() {
    if gtk::init().is_err() {
        println!("Hello, world!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let entry = FinishEntry::new();
    entry.connect_changed(|entry| {
        if let Some(value) = entry.value() {
            println!("{}: {}", value.abbrev(), value.full());
        } else {
            println!("None");
        }
    });
    entry.pwo().show_all();
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox.pack_start(&FinishEntry::prompt(), false, false, 0);
    hbox.pack_start(&entry.pwo(), false, false, 0);
    hbox.show_all();
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&hbox, false, false, 0);
    vbox.pack_start(
        &BasicPaintEditor::new(
            &[ScalarAttribute::Value, ScalarAttribute::Greyness],
            &vec![],
        )
        .pwo(),
        false,
        false,
        0,
    );
    let graticule = GtkGraticule::new(
        &[(
            "add",
            "Add",
            None,
            "Add the selected colour to the colour mixer",
            GtkGraticule::HAS_CHOSEN_ITEM,
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
    let rgbs: Vec<RGB> = RGB::PRIMARIES
        .iter()
        .chain(RGB::SECONDARIES.iter())
        .chain(RGB::GREYS.iter())
        .map(|x| *x)
        .collect();
    let list = RGBList::new(
        &rgbs,
        &[
            ScalarAttribute::Value,
            ScalarAttribute::Warmth,
            ScalarAttribute::Chroma,
        ],
    );
    vbox.pack_start(&list.pwo(), true, true, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
