// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{error, fmt, io};

pub mod factory;
pub mod icon_image;
pub mod list;
pub mod mixer;
pub mod series;
pub mod spec_edit;
pub mod storage;

pub mod characteristics {
    use std::{cell::RefCell, rc::Rc};

    use pw_gix::{
        gtk,
        gtk::{ComboBoxExt, ComboBoxTextExt},
        wrapper::*,
    };

    pub use apaint::characteristics::{
        CharacteristicIfce, CharacteristicType, Finish, Fluorescence, Metallicness, Permanence,
        Transparency,
    };

    type ChangeCallback<T> = Box<dyn Fn(&T)>;

    #[derive(PWO)]
    pub struct CharacteristicEntry<C: 'static + CharacteristicIfce> {
        combo_box_text: gtk::ComboBoxText,
        callbacks: RefCell<Vec<ChangeCallback<Self>>>,
        marker: std::marker::PhantomData<C>,
    }

    impl<C: CharacteristicIfce> CharacteristicEntry<C> {
        pub fn new() -> Rc<Self> {
            let combo_box_text = gtk::ComboBoxText::new();
            for str_value in C::str_values().iter() {
                combo_box_text.append_text(str_value);
            }
            combo_box_text.set_id_column(0);
            let ce = Rc::new(Self {
                combo_box_text,
                callbacks: RefCell::new(vec![]),
                marker: std::marker::PhantomData,
            });
            ce.set_value(None);
            let ce_clone = Rc::clone(&ce);
            ce.combo_box_text.connect_changed(move |_| {
                for callback in ce_clone.callbacks.borrow().iter() {
                    callback(&ce_clone);
                }
            });
            ce
        }

        pub fn label(&self) -> gtk::Label {
            gtk::Label::new(Some(C::NAME))
        }

        pub fn prompt(&self, align: gtk::Align) -> gtk::Label {
            gtk::LabelBuilder::new()
                .label(C::PROMPT)
                .halign(align)
                .build()
        }

        pub fn value(&self) -> C {
            if let Some(text) = self.combo_box_text.get_active_text() {
                C::from_str(&text).expect("all strings should be valid")
            } else {
                C::default()
            }
        }

        pub fn set_value(&self, new_value: Option<C>) {
            let id = if let Some(new_value) = new_value {
                new_value.full()
            } else {
                C::default().full()
            };
            self.combo_box_text.set_active_id(Some(id));
        }

        pub fn connect_changed<F: Fn(&Self) + 'static>(&self, f: F) {
            self.callbacks.borrow_mut().push(Box::new(f))
        }
    }

    pub type FinishEntry = CharacteristicEntry<Finish>;
    pub type TransparencyEntry = CharacteristicEntry<Transparency>;
    pub type PermanenceEntry = CharacteristicEntry<Permanence>;
    pub type FluorescenceEntry = CharacteristicEntry<Fluorescence>;
    pub type MetallicnessEntry = CharacteristicEntry<Metallicness>;
}

pub mod colour {
    pub use colour_math_gtk::{colour::*, coloured::*};

    pub trait PartsColour:
        colour_math::ColourIfce + apaint::TooltipText + apaint::LabelText + std::cmp::Ord + 'static
    {
    }

    use apaint::mixtures::Mixture;
    use apaint::series::*;

    impl PartsColour for SeriesPaint {}
    impl PartsColour for Mixture {}
}

pub mod window {
    use std::{cell::Cell, rc::Rc};

    use pw_gix::{
        gdk, gdk_pixbuf,
        gtk::{self, prelude::*},
        gtkx::window::RememberGeometry,
        wrapper::*,
    };

    #[derive(PWO)]
    pub struct PersistentWindowButton {
        button: gtk::Button,
        window: gtk::Window,
        is_iconified: Cell<bool>,
    }

    pub struct PersistentWindowButtonBuilder {
        button: gtk::Button,
        window: gtk::Window,
        is_iconified: Cell<bool>,
    }

    impl Default for PersistentWindowButtonBuilder {
        fn default() -> Self {
            Self {
                button: gtk::ButtonBuilder::new().build(),
                window: gtk::WindowBuilder::new().destroy_with_parent(true).build(),
                is_iconified: Cell::new(false),
            }
        }
    }

    impl PersistentWindowButtonBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn icon<P: IsA<gtk::Widget>>(self, image: &P) -> Self {
            self.button.set_image(Some(image));
            self
        }

        pub fn label(self, label: &str) -> Self {
            self.button.set_label(label);
            self
        }

        pub fn tooltip_text(self, text: &str) -> Self {
            self.button.set_tooltip_text(Some(text));
            self
        }

        pub fn window_title(self, title: &str) -> Self {
            self.window.set_title(title);
            self
        }

        pub fn window_icon(self, icon: &gdk_pixbuf::Pixbuf) -> Self {
            self.window.set_icon(Some(icon));
            self
        }

        pub fn window_child<P: IsA<gtk::Widget>>(self, widget: &P) -> Self {
            self.window.add(widget);
            self
        }

        pub fn window_geometry(
            self,
            saved_geometry_key: Option<&str>,
            default_size: (i32, i32),
        ) -> Self {
            if let Some(saved_geometry_key) = saved_geometry_key {
                self.window
                    .set_geometry_from_recollections(saved_geometry_key, default_size);
            } else {
                self.window
                    .set_default_geometry(default_size.0, default_size.1);
            }
            self
        }

        pub fn build(self) -> Rc<PersistentWindowButton> {
            let pwb = Rc::new(PersistentWindowButton {
                button: self.button,
                window: self.window,
                is_iconified: self.is_iconified,
            });

            pwb.window.connect_delete_event(|w, _| {
                w.hide_on_delete();
                gtk::Inhibit(true)
            });

            let pwb_c = Rc::clone(&pwb);
            pwb.window.connect_window_state_event(move |_, event| {
                let state = event.get_new_window_state();
                pwb_c
                    .is_iconified
                    .set(state.contains(gdk::WindowState::ICONIFIED));
                gtk::Inhibit(false)
            });

            let pwb_c = Rc::clone(&pwb);
            pwb.button.connect_clicked(move |_| {
                // NB: diconify() is unreliable due to window manager interference
                if pwb_c.window.get_visible() && !pwb_c.is_iconified.get() {
                    pwb_c.window.hide();
                } else {
                    pwb_c.window.present();
                }
            });

            pwb
        }
    }
}

#[derive(Debug)]
pub enum Error {
    APaintError(apaint::Error),
    IOError(io::Error),
    DuplicateFile(String),
    GeneralError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::APaintError(err) => write!(f, "Error: {}.", err),
            Error::IOError(err) => write!(f, "Error: {}.", err),
            Error::DuplicateFile(string) => write!(f, "Error: {}.", string),
            Error::GeneralError(string) => write!(f, "Error: {}.", string),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::APaintError(err) => Some(err),
            Error::IOError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<apaint::Error> for Error {
    fn from(err: apaint::Error) -> Self {
        Error::APaintError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}
