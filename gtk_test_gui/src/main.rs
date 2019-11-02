// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::wrapper::*;

use apaint::characteristics::CharacteristicIfce;

type FinishEntry =
    apaint_gtk::characteristics::CharacteristicEntry<apaint::characteristics::Finish>;

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
    win.add(&hbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
