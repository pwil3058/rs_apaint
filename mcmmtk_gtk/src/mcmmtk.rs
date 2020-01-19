// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{process::Command, rc::Rc};

use gtk::prelude::*;

use pw_gix::{
    gdk_pixbufx::viewer::PixbufViewBuilder, gtkx::window::RememberGeometry, sample, wrapper::*,
};

use apaint_gtk::characteristics::CharacteristicType;
use apaint_gtk::colour::ScalarAttribute;
use apaint_gtk::mixer::targeted::TargetedPaintMixer;

#[derive(PWO, Wrapper)]
pub struct ModellersColourMixerMatcherTK {
    vbox: gtk::Box,
    mixer: Rc<TargetedPaintMixer>,
}

impl ModellersColourMixerMatcherTK {
    pub fn new() -> Rc<Self> {
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
        let mcmmtk = Rc::new(Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            mixer,
        });
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        mcmmtk.vbox.pack_start(&hbox, false, false, 0);

        let stack = gtk::StackBuilder::new().build();
        stack.add_titled(&mcmmtk.mixer.pwo(), "mixer", "Mixer");
        mcmmtk.vbox.pack_start(&stack, true, true, 0);
        let stack_switcher = gtk::StackSwitcherBuilder::new()
            .tooltip_text("Select mode.")
            .stack(&stack)
            .build();
        hbox.pack_start(&stack_switcher, true, true, 0);

        let seperator = gtk::SeparatorBuilder::new().build();
        hbox.pack_start(&seperator, false, false, 0);

        let button = gtk::Button::new_with_label("PDF Viewer");
        hbox.pack_start(&button, false, false, 0);
        let mcmmtk_c = Rc::clone(&mcmmtk);
        button.connect_clicked(move |_| mcmmtk_c.launch_pdf_viewer());

        let button = gtk::Button::new_with_label("Image Viewer");
        hbox.pack_start(&button, false, false, 0);
        button.connect_clicked(move |_| launch_image_viewer());

        if sample::screen_sampling_available() {
            let btn = gtk::Button::new_with_label("Take Sample");
            btn.set_tooltip_text(Some("Take a sample of a portion of the screen"));
            let mcmmtk_c = Rc::clone(&mcmmtk);
            btn.connect_clicked(move |_| {
                if let Err(err) = sample::take_screen_sample() {
                    mcmmtk_c.report_error("Failure", &err);
                }
            });
            hbox.pack_start(&btn, false, false, 0);
        }

        mcmmtk.vbox.show_all();

        mcmmtk
    }

    pub fn ok_to_quit(&self) -> bool {
        self.ask_confirm_action("OK to quit?", None)
    }

    fn launch_pdf_viewer(&self) {
        // TODO: make pdf viewer configurable
        let viewer = "xreader";
        if let Err(err) = Command::new(viewer).spawn() {
            let msg = format!("Error running \"{}\"", viewer);
            self.report_error(&msg, &err);
        }
    }
}

fn launch_image_viewer() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_geometry_from_recollections("mcmmtk_gtk::image_viewer", (200, 200));
    window.set_destroy_with_parent(true);
    window.set_title("mcmmtk_gtk: Image Viewer");

    let view = PixbufViewBuilder::new().load_last_image(true).build();
    window.add(&view.pwo());
    window.show_all();

    window.present();
}
