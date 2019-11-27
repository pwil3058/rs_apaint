// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

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

    use apaint::characteristics::{CharacteristicIfce, Finish};
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

        pub fn label() -> gtk::Label {
            gtk::Label::new(Some(C::NAME))
        }

        pub fn prompt() -> gtk::Label {
            gtk::Label::new(Some(C::PROMPT))
        }

        pub fn value(&self) -> Option<C> {
            if let Some(text) = self.combo_box_text.get_active_text() {
                match C::from_str(&text) {
                    Ok(c) => Some(c),
                    Err(_) => panic!("all strings should be valid"),
                }
            } else {
                None
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

pub mod attributes;
pub mod colour_edit;
pub mod factory;
pub mod hue_wheel;
pub mod icon_image;
pub mod list;
pub mod spec_edit;
