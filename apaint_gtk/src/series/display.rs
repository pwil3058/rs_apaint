// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{collections::BTreeMap, rc::Rc};

use pw_gix::{
    gtk::{self, prelude::*},
    gtkx::dialog::dialog_user::TopGtkWindow,
    sav_state::{ChangedCondnsNotifier, ConditionalWidgetsBuilder},
    wrapper::*,
};

use colour_math::{ColourBasics, ScalarAttribute};
#[cfg(feature = "targeted_mixtures")]
use colour_math_gtk::attributes::ColourAttributeDisplayStack;
use colour_math_gtk::attributes::ColourAttributeDisplayStackBuilder;

use apaint::{characteristics::CharacteristicType, series::SeriesPaint, BasicPaintIfce};

use crate::colour::{Colourable, GdkColour, HCV};
use crate::series::PaintActionCallback;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(PWO)]
pub struct PaintDisplay {
    vbox: gtk::Box,
    paint: Rc<SeriesPaint>,
    #[cfg(feature = "targeted_mixtures")]
    target_label: gtk::Label,
    #[cfg(feature = "targeted_mixtures")]
    cads: Rc<ColourAttributeDisplayStack>,
}

impl PaintDisplay {
    #[cfg(feature = "targeted_mixtures")]
    pub fn set_target_colour(&self, new_target: Option<&impl GdkColour>) {
        if let Some(colour) = new_target {
            self.target_label.set_label("Current Target");
            self.target_label.set_widget_colour(colour);
            self.cads.set_target_colour(Some(colour));
        } else {
            self.target_label.set_label("");
            self.target_label.set_widget_colour(&self.paint.hcv());
            self.cads.set_target_colour(Option::<&HCV>::None);
        };
    }

    pub fn paint(&self) -> &Rc<SeriesPaint> {
        &self.paint
    }
}

#[derive(Default)]
pub struct PaintDisplayBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    #[cfg(feature = "targeted_mixtures")]
    target_colour: Option<HCV>,
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

    #[cfg(feature = "targeted_mixtures")]
    pub fn target_colour(&mut self, target_colour: Option<&impl GdkColour>) -> &mut Self {
        self.target_colour = if let Some(target_colour) = target_colour {
            Some(target_colour.hcv())
        } else {
            None
        };
        self
    }

    pub fn build(&self, paint: &Rc<SeriesPaint>) -> PaintDisplay {
        let hcv = paint.hcv();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let label = gtk::LabelBuilder::new().label(paint.id()).build();
        label.set_widget_colour(&hcv);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.name().unwrap_or(""))
            .build();
        label.set_widget_colour(&hcv);
        vbox.pack_start(&label, false, false, 0);

        let label = gtk::LabelBuilder::new()
            .label(paint.notes().unwrap_or(""))
            .build();
        label.set_widget_colour(&hcv);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.series_name())
            .build();
        label.set_widget_colour(&hcv);
        vbox.pack_start(&label, false, false, 0);

        let series_id = paint.series_id();
        let label = gtk::LabelBuilder::new()
            .label(series_id.proprietor())
            .build();
        label.set_widget_colour(&hcv);
        vbox.pack_start(&label, false, false, 0);

        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(&self.attributes)
            .build();
        cads.set_colour(Some(&hcv));

        #[cfg(feature = "targeted_mixtures")]
        let target_label = if let Some(target_colour) = self.target_colour {
            let label = gtk::LabelBuilder::new().label("Target").build();
            label.set_widget_colour(&target_colour);
            cads.set_target_colour(Some(&target_colour));
            label
        } else {
            let label = gtk::LabelBuilder::new().build();
            label.set_widget_colour(&hcv);
            label
        };
        #[cfg(feature = "targeted_mixtures")]
        vbox.pack_start(&target_label, true, true, 0);
        vbox.pack_start(cads.pwo(), true, true, 0);

        for characteristic_type in self.characteristics.iter() {
            let value = paint.characteristic(*characteristic_type).full();
            let label = gtk::LabelBuilder::new().label(value).build();
            label.set_widget_colour(&hcv);
            vbox.pack_start(&label, false, false, 0);
        }
        vbox.show_all();

        PaintDisplay {
            vbox,
            paint: Rc::clone(paint),
            #[cfg(feature = "targeted_mixtures")]
            target_label,
            #[cfg(feature = "targeted_mixtures")]
            cads,
        }
    }
}

struct PaintDisplayDialog {
    dialog: gtk::Dialog,
    #[cfg(feature = "targeted_mixtures")]
    display: PaintDisplay,
}

pub struct PaintDisplayDialogManager<W: TopGtkWindow> {
    caller: W,
    buttons: Vec<(u16, &'static str, Option<&'static str>, u64)>,
    button_callbacks: RefCell<HashMap<u16, Vec<PaintActionCallback>>>,
    paint_display_builder: RefCell<PaintDisplayBuilder>,
    conditional_widgets_builder: ConditionalWidgetsBuilder,
    dialogs: RefCell<BTreeMap<Rc<SeriesPaint>, PaintDisplayDialog>>,
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
            Inhibit(true)
        });
        dialog
    }

    #[cfg(feature = "targeted_mixtures")]
    pub fn set_target_colour(&self, colour: Option<&impl GdkColour>) {
        self.paint_display_builder
            .borrow_mut()
            .target_colour(colour);
        for pdd in self.dialogs.borrow().values() {
            pdd.display.set_target_colour(colour);
        }
    }

    fn inform_button_action(&self, action: u16, paint: Rc<SeriesPaint>) {
        let button_callbacks = self.button_callbacks.borrow();
        for callback in button_callbacks
            .get(&action)
            .expect("programmer error")
            .iter()
        {
            callback(Rc::clone(&paint))
        }
    }

    pub fn connect_action_button<F: Fn(Rc<SeriesPaint>) + 'static>(
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
    fn display_paint(&self, paint: &Rc<SeriesPaint>);
}

impl<W: TopGtkWindow + 'static> DisplayPaint for Rc<PaintDisplayDialogManager<W>> {
    fn display_paint(&self, paint: &Rc<SeriesPaint>) {
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
                .pack_start(display.pwo(), true, true, 0);
            let paint_c = Rc::clone(paint);
            let self_c = Rc::clone(self);
            dialog.connect_response(move |_, response| {
                if let gtk::ResponseType::Other(code) = response {
                    self_c.inform_button_action(code, Rc::clone(&paint_c));
                }
            });
            #[cfg(feature = "targeted_mixtures")]
            let pdd = PaintDisplayDialog { dialog, display };
            #[cfg(not(feature = "targeted_mixtures"))]
            let pdd = PaintDisplayDialog { dialog };
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
    target_colour: Option<HCV>,
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
            target_colour: None,
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

    pub fn target_colour(&mut self, target_colour: &impl GdkColour) -> &mut Self {
        self.target_colour = Some(target_colour.hcv());
        self
    }

    pub fn build(&self) -> Rc<PaintDisplayDialogManager<W>> {
        let mut paint_display_builder = PaintDisplayBuilder::new();
        paint_display_builder
            .attributes(&self.attributes)
            .characteristics(&self.characteristics);
        #[cfg(feature = "targeted_mixtures")]
        if let Some(target_colour) = self.target_colour {
            paint_display_builder.target_colour(Some(&target_colour));
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
