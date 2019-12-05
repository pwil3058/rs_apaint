// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{error, fmt};

pub const SAV_HAS_CHOSEN_ITEM: u64 = 1 << 0;

pub mod angles {
    pub use normalised_angles;

    pub type Angle = normalised_angles::Angle<f64>;
    pub type Degrees = normalised_angles::Degrees<f64>;
    pub type Radians = normalised_angles::Radians<f64>;
}

pub mod characteristics {
    use std::cell::RefCell;
    use std::rc::Rc;

    use apaint_gtk_boilerplate::PWO;
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
                match C::from_str(&text) {
                    Ok(c) => c,
                    Err(_) => panic!("all strings should be valid"),
                }
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

    pub type Colour = colour_math::Colour<f64>;
    pub type Hue = colour_math::hue::Hue<f64>;
    pub type RGB = colour_math::rgb::RGB<f64>;
    pub type RGBManipulator = colour_math::manipulator::RGBManipulator<f64>;

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

pub mod attributes;
pub mod colour_edit;
pub mod factory;
pub mod hue_wheel;
pub mod icon_image;
pub mod list;
pub mod series;
pub mod spec_edit;

#[derive(Debug)]
pub enum Error {
    APaintError(apaint::Error),
    GeneralError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::APaintError(err) => write!(f, "Error: {}.", err),
            Error::GeneralError(string) => write!(f, "Error: {}.", string),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::APaintError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<apaint::Error> for Error {
    fn from(err: apaint::Error) -> Self {
        Error::APaintError(err)
    }
}
