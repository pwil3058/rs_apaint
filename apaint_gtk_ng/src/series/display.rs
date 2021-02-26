// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{collections::BTreeMap, rc::Rc};

use pw_gix::{
    gtk::{self, prelude::*},
    gtkx::dialog::dialog_user::TopGtkWindow,
    sav_state::{ChangedCondnsNotifier, ConditionalWidgetsBuilder},
    wrapper::*,
};

use colour_math_gtk_ng::attributes::{
    ColourAttributeDisplayStack, ColourAttributeDisplayStackBuilder,
};
use colour_math_ng::{ColourBasics, ScalarAttribute};

use apaint_ng::{characteristics::CharacteristicType, series::SeriesPaint, BasicPaintIfce};

use crate::colour::{Colourable, RGB};
use crate::series::PaintActionCallback;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(PWO)]
pub struct PaintDisplay {
    vbox: gtk::Box,
    paint: Rc<SeriesPaint<f64>>,
    target_label: gtk::Label,
    cads: ColourAttributeDisplayStack,
}

impl PaintDisplay {
    pub fn set_target(&self, new_target: Option<&RGB<f64>>) {
        if let Some(rgb) = new_target {
            self.target_label.set_label("Current Target");
            self.target_label.set_widget_colour_rgb(rgb);
            self.cads.set_target_colour(Some(rgb));
        } else {
            self.target_label.set_label("");
            self.target_label.set_widget_colour_rgb(&self.paint.rgb());
            self.cads.set_target_colour(Option::<&RGB<f64>>::None);
        };
    }

    pub fn paint(&self) -> &Rc<SeriesPaint<f64>> {
        &self.paint
    }
}

#[derive(Default)]
pub struct PaintDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB<f64>>,
}

impl PaintDisplayBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
        self.attributes = attributes.to_vec();
        self
    }

    pub fn characteristics(&mut self, characteristics: &[CharacteristicType]) -> &mut Self {
        self.characteristics = characteristics.to_vec();
        self
    }

    pub fn target_rgb(&mut self, target_rgb: Option<&RGB<f64>>) -> &mut Self {
        self.target_rgb = if let Some(target_rgb) = target_rgb {
            Some(*target_rgb)
        } else {
            None
        };
        self
    }

    pub fn build(&self, paint: &Rc<SeriesPaint<f64>>) -> PaintDisplay {
        let rgb = paint.rgb();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let label = gtk::LabelBuilder::new().label(paint.id()).build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.name().unwrap_or(""))
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.notes().unwrap_or(""))
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.series_name())
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.proprietor())
            .build();
        label.set_widget_colour_rgb(&rgb);
        vbox.pack_start(&label, false, false, 0);

        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(&self.attributes)
            .build();
        cads.set_colour(Some(&rgb));
        let target_label = if let Some(target_rgb) = self.target_rgb {
            let label = gtk::LabelBuilder::new().label("Target").build();
            label.set_widget_colour_rgb(&target_rgb);
            cads.set_target_colour(Some(&target_rgb));
            label
        } else {
            let label = gtk::LabelBuilder::new().build();
            label.set_widget_colour_rgb(&rgb);
            label
        };
        vbox.pack_start(&target_label, true, true, 0);
        vbox.pack_start(&cads.pwo(), true, true, 0);

        for characteristic_type in self.characteristics.iter() {
            let value = paint.characteristic(*characteristic_type).full();
            let label = gtk::LabelBuilder::new().label(&value).build();
            label.set_widget_colour_rgb(&rgb);
            vbox.pack_start(&label, false, false, 0);
        }
        vbox.show_all();

        PaintDisplay {
            vbox,
            paint: Rc::clone(paint),
            target_label,
            cads,
        }
    }
}

struct PaintDisplayDialog {
    dialog: gtk::Dialog,
    display: PaintDisplay,
}

pub struct PaintDisplayDialogManager<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(u16, &'static str, Option<&'static str>, u64)>,
    button_callbacks: RefCell<HashMap<u16, Vec<PaintActionCallback>>>,
    paint_display_builder: RefCell<PaintDisplayBuilder>,
    conditional_widgets_builder: ConditionalWidgetsBuilder,
    dialogs: RefCell<BTreeMap<Rc<SeriesPaint<f64>>, PaintDisplayDialog>>,
}

impl<W: TopGtkWindow> PaintDisplayDialogManager<W> {
    fn new_dialog(&self) -> gtk::Dialog {
        let dialog = gtk::DialogBuilder::new().build();
        if let Some(parent) = self.caller.get_toplevel_gtk_window() {
            dialog.set_transient_for(Some(&parent));
        }
        // TODO: think about removal from map as an optional action to hiding
        dialog.connect_delete_event(|d, _| {
            d.hide_on_delete();
            gtk::Inhibit(true)
        });
        dialog
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB<f64>>) {
        self.paint_display_builder.borrow_mut().target_rgb(rgb);
        for pdd in self.dialogs.borrow().values() {
            pdd.display.set_target(rgb);
        }
    }

    fn inform_button_action(&self, action: u16, paint: Rc<SeriesPaint<f64>>) {
        let button_callbacks = self.button_callbacks.borrow();
        for callback in button_callbacks
            .get(&action)
            .expect("programmer error")
            .iter()
        {
            callback(Rc::clone(&paint))
        }
    }

    pub fn connect_action_button<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(
        &self,
        action: u16,
        callback: F,
    ) {
        self.button_callbacks
            .borrow_mut()
            .get_mut(&action)
            .expect("programmer error")
            .push(Box::new(callback));
    }
}

pub trait DisplayPaint {
    fn display_paint(&self, paint: &Rc<SeriesPaint<f64>>);
}

impl<W: TopGtkWindow + 'static> DisplayPaint for Rc<PaintDisplayDialogManager<W>> {
    fn display_paint(&self, paint: &Rc<SeriesPaint<f64>>) {
        if !self.dialogs.borrow().contains_key(paint) {
            let dialog = self.new_dialog();
            let display = self.paint_display_builder.borrow().build(paint);
            let managed_buttons = self.conditional_widgets_builder.build::<u16, gtk::Widget>();
            for (response, label, tooltip_text, condns) in self.buttons.iter() {
                let button = dialog.add_button(label, gtk::ResponseType::Other(*response));
                button.set_tooltip_text(*tooltip_text);
                managed_buttons.add_widget(*response, &button, *condns);
            }
            dialog
                .get_content_area()
                .pack_start(&display.pwo(), true, true, 0);
            let paint_c = Rc::clone(paint);
            let self_c = Rc::clone(self);
            dialog.connect_response(move |_, response| {
                if let gtk::ResponseType::Other(code) = response {
                    self_c.inform_button_action(code, Rc::clone(&paint_c));
                }
            });
            let pdd = PaintDisplayDialog { dialog, display };
            self.dialogs.borrow_mut().insert(Rc::clone(paint), pdd);
        };
        let dialogs = self.dialogs.borrow();
        let pdd = dialogs.get(paint).expect("we just put it there");
        pdd.dialog.present();
    }
}

pub struct PaintDisplayDialogManagerBuilder<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(u16, &'static str, Option<&'static str>, u64)>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: Option<RGB<f64>>,
    change_notifier: Rc<ChangedCondnsNotifier>,
}

impl<W: TopGtkWindow + Clone> PaintDisplayDialogManagerBuilder<W> {
    pub fn new(caller: &W) -> Self {
        let change_notifier = Rc::new(ChangedCondnsNotifier::default());
        Self {
            caller: caller.clone(),
            buttons: vec![],
            attributes: vec![],
            characteristics: vec![],
            target_rgb: None,
            change_notifier,
        }
    }

    pub fn attributes(&mut self, attributes: &[ScalarAttribute]) -> &mut Self {
        self.attributes = attributes.to_vec();
        self
    }

    pub fn characteristics(&mut self, characteristics: &[CharacteristicType]) -> &mut Self {
        self.characteristics = characteristics.to_vec();
        self
    }

    pub fn buttons(
        &mut self,
        buttons: &[(u16, &'static str, Option<&'static str>, u64)],
    ) -> &mut Self {
        self.buttons = buttons.to_vec();
        self
    }

    pub fn change_notifier(&mut self, change_notifier: &Rc<ChangedCondnsNotifier>) -> &mut Self {
        self.change_notifier = Rc::clone(change_notifier);
        self
    }

    pub fn build(&self) -> Rc<PaintDisplayDialogManager<W>> {
        let mut paint_display_builder = PaintDisplayBuilder::new();
        paint_display_builder
            .attributes(&self.attributes)
            .characteristics(&self.characteristics);
        if let Some(target_rgb) = self.target_rgb {
            paint_display_builder.target_rgb(Some(&target_rgb));
        }
        let mut hash_map: HashMap<u16, Vec<PaintActionCallback>> = HashMap::new();
        for (id, _, _, _) in self.buttons.iter() {
            hash_map.insert(*id, vec![]);
        }
        let mut conditional_widgets_builder = ConditionalWidgetsBuilder::new();
        conditional_widgets_builder.change_notifier(&self.change_notifier);
        Rc::new(PaintDisplayDialogManager {
            caller: self.caller.clone(),
            buttons: self.buttons.clone(),
            button_callbacks: RefCell::new(hash_map),
            paint_display_builder: RefCell::new(paint_display_builder),
            conditional_widgets_builder,
            dialogs: RefCell::new(BTreeMap::new()),
        })
    }
}
