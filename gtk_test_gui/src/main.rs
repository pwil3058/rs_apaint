// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::wrapper::*;

use apaint::characteristics::CharacteristicIfce;

use apaint_gtk::attributes::artist_cads;
use apaint_gtk::characteristics::FinishEntry;
use apaint_gtk::colour::{ScalarAttribute, RGB};
use apaint_gtk::colour_edit::ColourEditor;
use apaint_gtk::graticule::GtkGraticule;

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
        &ColourEditor::new(&artist_cads(), &vec![]).pwo(),
        false,
        false,
        0,
    );
    let graticule = GtkGraticule::new(
        &vec![],
        &[
            ScalarAttribute::Value,
            ScalarAttribute::Chroma,
            ScalarAttribute::Warmth,
        ],
    );
    for rgb in RGB::PRIMARIES.iter() {
        graticule.add_item(*rgb);
    }
    for rgb in RGB::SECONDARIES.iter() {
        graticule.add_item(*rgb);
    }
    for rgb in RGB::GREYS.iter() {
        graticule.add_item(*rgb);
    }
    vbox.pack_start(&graticule.pwo(), true, true, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
