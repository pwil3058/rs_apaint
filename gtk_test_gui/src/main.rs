// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::{ContainerExt, WidgetExt};

use pw_gix::wrapper::*;

use apaint::characteristics::CharacteristicIfce;

fn main() {
    if gtk::init().is_err() {
        println!("Hello, world!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let entry =
        apaint_gtk::characteristics::CharacteristicEntry::<apaint::characteristics::Finish>::new();
    entry.connect_changed(|entry| {
        if let Some(value) = entry.value() {
            println!("{}: {}", value.abbrev(), value.full());
        } else {
            println!("None");
        }
    });
    entry.pwo().show_all();
    win.add(&entry.pwo());
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
