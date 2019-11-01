// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::{ContainerExt, DialogExt, GtkWindowExt, WidgetExt};

use pw_gix::wrapper::*;

fn main() {
    if gtk::init().is_err() {
        println!("Hello, world!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    let entry =
        apaint_gtk::characteristics::CharacteristicEntry::<apaint::characteristics::Finish>::new();
    entry.pwo().show_all();
    win.add(&entry.pwo());
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
