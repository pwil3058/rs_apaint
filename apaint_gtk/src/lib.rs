// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{error, fmt, io};

pub mod angles {
    pub use normalised_angles;

    pub type Angle = normalised_angles::Angle<f64>;
    pub type Degrees = normalised_angles::Degrees<f64>;
    pub type Radians = normalised_angles::Radians<f64>;
}

pub mod characteristics {
    use std::{cell::RefCell, rc::Rc};

    use pw_gix::wrapper::*;

    pub use apaint::characteristics::{
        CharacteristicIfce, CharacteristicType, Finish, Fluorescence, Metallicness, Permanence,
        Transparency,
    };
    use gtk::{ComboBoxExt, ComboBoxTextExt};

    #[derive(PWO)]
    pub struct CharacteristicEntry<C: 'static + CharacteristicIfce> {
        combo_box_text: gtk::ComboBoxText,
        callbacks: RefCell<Vec<Box<dyn Fn(&Self)>>>,
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
    pub use colour_math::{
        rgb::{RGBError, RGB16, RGB8},
        ColourInterface, ScalarAttribute,
    };
    use gdk;
    pub use normalised_angles;

    pub type Colour = colour_math::Colour<f64>;
    pub type Hue = colour_math::hue::Hue<f64>;
    pub type RGB = colour_math::rgb::RGB<f64>;
    pub type RGBManipulator = colour_math::manipulator::RGBManipulator<f64>;
    pub type Degrees = normalised_angles::Degrees<f64>;
    pub type Radians = normalised_angles::Radians<f64>;
    pub type Angle = normalised_angles::Angle<f64>;

    pub trait GdkConvert {
        fn into_gdk_rgba(&self) -> gdk::RGBA;
    }

    impl GdkConvert for RGB {
        fn into_gdk_rgba(&self) -> gdk::RGBA {
            pw_gix::colour::rgba_from_rgb(*self)
        }
    }
}

pub mod managed_menu {
    #[derive(Debug, Clone)]
    pub struct MenuItemSpec {
        name: String,
        label: String,
        image: Option<gtk::Image>,
        tooltip: String,
        condns: u64,
    }

    impl MenuItemSpec {
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn label(&self) -> &str {
            &self.label
        }

        pub fn image(&self) -> Option<&gtk::Image> {
            if let Some(ref image) = self.image {
                Some(image)
            } else {
                None
            }
        }

        pub fn tooltip(&self) -> &str {
            &self.tooltip
        }

        pub fn condns(&self) -> u64 {
            self.condns
        }
    }

    impl From<(&str, &str, Option<gtk::Image>, &str, u64)> for MenuItemSpec {
        fn from(tuple_: (&str, &str, Option<gtk::Image>, &str, u64)) -> Self {
            Self {
                name: tuple_.0.to_string(),
                label: tuple_.1.to_string(),
                image: tuple_.2,
                tooltip: tuple_.3.to_string(),
                condns: tuple_.4,
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::managed_menu::MenuItemSpec;

        #[test]
        fn test_list_initialization() {
            let list: &[MenuItemSpec] = &[
                ("test", "Test", None, "testing", 0).into(),
                ("test1", "Test1", None, "testing", 0).into(),
            ];
            let _v: Vec<MenuItemSpec> = list.to_vec();
        }
    }
}

pub mod window {
    use std::{cell::Cell, rc::Rc};

    use gtk::prelude::*;

    use pw_gix::wrapper::*;

    use pw_gix::gtkx::window::RememberGeometry;

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

pub mod attributes;
pub mod colour_edit;
pub mod factory;
pub mod hue_wheel;
pub mod icon_image;
pub mod list;
pub mod mixer;
pub mod series;
pub mod spec_edit;
pub mod storage;

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
