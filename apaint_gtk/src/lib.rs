// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

#[macro_use]
extern crate pw_gix;

pub mod characteristics {
    use std::rc::Rc;

    use pw_gix::wrapper::*;

    use apaint::characteristics::CharacteristicIfce;
    use gtk::{ComboBoxExt, ComboBoxExtManual, ComboBoxTextExt, WidgetExt};

    pub struct CharacteristicEntry<C: CharacteristicIfce> {
        combo_box_text: gtk::ComboBoxText,
        marker: std::marker::PhantomData<C>,
    }

    impl_widget_wrapper!(
        combo_box_text: gtk::ComboBoxText,
        CharacteristicEntry<C> where C: CharacteristicIfce
    );

    impl<C: CharacteristicIfce> CharacteristicEntry<C> {
        pub fn new() -> Rc<Self> {
            let commbo_box_text = gtk::ComboBoxText::new();
            for str_value in C::str_values().iter() {
                commbo_box_text.append_text(str_value);
            }
            Rc::new(Self {
                combo_box_text: commbo_box_text,
                marker: std::marker::PhantomData,
            })
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
            if let Some(new_value) = new_value {
                let full = new_value.full();
                self.combo_box_text.set_active_id(Some(full));
            } else {
                self.combo_box_text.set_active_id(None);
            }
        }
    }
}
