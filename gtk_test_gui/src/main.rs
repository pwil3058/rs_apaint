// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::recollections;
use pw_gix::wrapper::*;

use apaint::basic_paint::BasicPaint;

use apaint::characteristics::CharacteristicType;
use apaint::series::PaintSeries;
use apaint_gtk::colour::RGB;
use apaint_gtk::series::{SeriesBinder, SeriesPage};
use apaint_gtk::{colour::ScalarAttribute, factory::BasicPaintFactory};
use std::fs::File;

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
    let mut file = File::open("./test_saved_file.json").unwrap();
    let paint_series = PaintSeries::<f64, BasicPaint<f64>>::read(&mut file).unwrap();
    let page = SeriesPage::new(
        paint_series,
        &[("test", "Test", None, "testing", 0)],
        &[ScalarAttribute::Value, ScalarAttribute::Greyness],
        &[
            CharacteristicType::Finish,
            CharacteristicType::Transparency,
            CharacteristicType::Fluorescence,
            CharacteristicType::Metallicness,
        ],
    );
    page.connect_popup_menu_item("test", |sid, id| println!("{:?}:{:?}", sid, id));
    page.set_target_rgb(Some(&RGB::GREEN));
    vbox.pack_start(&page.pwo(), true, true, 0);
    let binder = SeriesBinder::<BasicPaint<f64>>::new(
        &[("test", "Test", None, "testing", 0)],
        &[ScalarAttribute::Value, ScalarAttribute::Greyness],
        &[
            CharacteristicType::Finish,
            CharacteristicType::Transparency,
            CharacteristicType::Fluorescence,
            CharacteristicType::Metallicness,
        ],
    );
    let mut file = File::open("./test_saved_file.json").unwrap();
    let paint_series = PaintSeries::<f64, BasicPaint<f64>>::read(&mut file).unwrap();
    binder.add_series(paint_series);
    vbox.pack_start(&binder.pwo(), true, true, 0);
    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
