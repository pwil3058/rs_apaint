// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

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
            let ce = Rc::new(Self {
                combo_box_text,
                callbacks: RefCell::new(vec![]),
                marker: std::marker::PhantomData,
            });
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
            if let Some(new_value) = new_value {
                let full = new_value.full();
                self.combo_box_text.set_active_id(Some(full));
            } else {
                self.combo_box_text.set_active_id(None);
            }
        }

        pub fn connect_changed<F: Fn(&Self) + 'static>(&self, f: F) {
            self.callbacks.borrow_mut().push(Box::new(f))
        }
    }

    pub type FinishEntry = CharacteristicEntry<Finish>;
}
