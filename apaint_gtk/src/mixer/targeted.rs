// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;

use cairo;

use pw_gix::{
    cairox::*,
    gtkx::paned::RememberPosition,
    sav_state::{ConditionalWidgetGroups, WidgetStatesControlled, SAV_NEXT_CONDN},
    wrapper::*,
};

use colour_math::ScalarAttribute;

use apaint_gtk_boilerplate::PWO;

use apaint::{
    characteristics::CharacteristicType, colour_mix::ColourMixer, hue_wheel::MakeColouredShape,
    mixtures::MixedPaint, series::SeriesPaint,
};

use crate::{
    attributes::ColourAttributeDisplayStack,
    colour::RGB,
    hue_wheel::GtkHueWheel,
    icon_image::series_paint_image,
    list::{ColouredItemListView, PaintListHelper, PaintListRow},
    mixer::component::{PartsSpinButtonBox, RcPartsSpinButtonBox},
    series::PaintSeriesManager,
    window::PersistentWindowButtonBuilder,
};
use pw_gix::sav_state::MaskedCondns;

// TODO: modify PaintListRow for MixedPaint to included target RGB
impl PaintListRow for MixedPaint<f64> {}

#[derive(PWO)]
pub struct TargetedPaintEntry {
    vbox: gtk::Box,
    id_label: gtk::Label,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    cads: ColourAttributeDisplayStack,
    drawing_area: gtk::DrawingArea,
    mix_rgb: RefCell<Option<RGB>>,
    target_rgb: RefCell<Option<RGB>>,
}

impl TargetedPaintEntry {
    pub fn new(attributes: &[ScalarAttribute]) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let id_label = gtk::LabelBuilder::new().label("#???").build();
        let name_entry = gtk::EntryBuilder::new().build();
        let notes_entry = gtk::EntryBuilder::new().build();
        let cads = ColourAttributeDisplayStack::new(attributes);
        let drawing_area = gtk::DrawingAreaBuilder::new().height_request(100).build();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&id_label, false, false, 0);
        hbox.pack_start(&name_entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&gtk::Label::new(Some("Notes: ")), false, false, 0);
        hbox.pack_start(&notes_entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&cads.pwo(), false, false, 0);
        vbox.pack_start(&drawing_area, true, true, 0);
        vbox.show_all();
        let tpe = Rc::new(Self {
            vbox,
            id_label,
            name_entry,
            notes_entry,
            cads,
            drawing_area,
            mix_rgb: RefCell::new(None),
            target_rgb: RefCell::new(None),
        });

        let tpe_c = Rc::clone(&tpe);
        tpe.drawing_area.connect_draw(move |da, ctxt| {
            tpe_c.draw(da, ctxt);
            Inhibit(false)
        });

        tpe
    }

    pub fn draw(&self, drawing_area: &gtk::DrawingArea, cairo_context: &cairo::Context) {
        if let Some(ref rgb) = *self.mix_rgb.borrow() {
            cairo_context.set_source_colour_rgb(*rgb);
        } else {
            cairo_context.set_source_colour_rgb(RGB::BLACK);
        };
        cairo_context.paint();
        if let Some(ref rgb) = *self.target_rgb.borrow() {
            cairo_context.set_source_colour_rgb(*rgb);
            let width = drawing_area.get_allocated_width() as f64;
            let height = drawing_area.get_allocated_height() as f64;
            cairo_context.rectangle(width / 4.0, height / 4.0, width / 2.0, height / 2.0);
            cairo_context.fill();
        }
    }

    pub fn set_mix_rgb(&self, rgb: Option<&RGB>) {
        if let Some(rgb) = rgb {
            *self.mix_rgb.borrow_mut() = Some(*rgb);
            self.cads.set_colour(Some(rgb));
        } else {
            *self.mix_rgb.borrow_mut() = None;
            self.cads.set_colour(Option::<&RGB>::None);
        }
        self.drawing_area.queue_draw()
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        if let Some(rgb) = rgb {
            *self.target_rgb.borrow_mut() = Some(*rgb);
            self.cads.set_target_colour(Some(rgb));
        } else {
            *self.target_rgb.borrow_mut() = None;
            self.cads.set_target_colour(Option::<&RGB>::None);
        }
        self.drawing_area.queue_draw()
    }
}

#[derive(PWO)]
pub struct TargetedPaintMixer {
    vbox: gtk::Box,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    mix_entry: Rc<TargetedPaintEntry>,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    series_paint_spinner_box: Rc<PartsSpinButtonBox<SeriesPaint<f64>>>,
    paint_series_manager: Rc<PaintSeriesManager>,
}

impl TargetedPaintMixer {
    const SAV_HAS_COLOUR: u64 = SAV_NEXT_CONDN << 0;
    const SAV_HAS_TARGET: u64 = SAV_NEXT_CONDN << 1;
    const SAV_NOT_HAS_TARGET: u64 = SAV_NEXT_CONDN << 2;
    const HAS_TARGET_MASK: u64 = Self::SAV_HAS_TARGET + Self::SAV_NOT_HAS_TARGET;
    const SAV_HAS_NAME: u64 = SAV_NEXT_CONDN << 3;

    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let hue_wheel = GtkHueWheel::new(&[], attributes);
        let helper = PaintListHelper::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(&helper.column_types(), &helper.columns(), &[]);
        let mix_entry = TargetedPaintEntry::new(attributes);
        let series_paint_spinner_box =
            PartsSpinButtonBox::<SeriesPaint<f64>>::new("Paints", 4, true);
        let paint_series_manager = PaintSeriesManager::new(attributes, characteristics);
        let persistent_window_btn = PersistentWindowButtonBuilder::new()
            .icon(&series_paint_image(24))
            .window_child(&paint_series_manager.pwo())
            .window_title("Paint Series Manager")
            .window_geometry(Some("paint_series_manager"), (300, 200))
            .build();
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        button_box.pack_start(&persistent_window_btn.pwo(), false, false, 0);
        vbox.pack_start(&button_box, false, false, 0);
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.add1(&hue_wheel.pwo());
        paned.add2(&mix_entry.pwo());
        paned.set_position_from_recollections("basic paint factory h paned position", 200);
        vbox.pack_start(&paned, true, true, 0);
        let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
            WidgetStatesControlled::Sensitivity,
            None,
            None,
        );
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let accept_btn = gtk::ButtonBuilder::new().label("Accept").build();
        buttons.add_widget(
            "accept",
            &accept_btn,
            Self::SAV_HAS_COLOUR + Self::SAV_HAS_TARGET + Self::SAV_HAS_NAME,
        );
        button_box.pack_start(&accept_btn, true, true, 0);
        vbox.pack_start(&button_box, false, false, 0);
        vbox.pack_start(&series_paint_spinner_box.pwo(), false, false, 0);
        vbox.pack_start(&list_view.pwo(), true, true, 0);
        vbox.show_all();

        let tpm = Rc::new(Self {
            vbox,
            hue_wheel,
            list_view,
            mix_entry,
            buttons,
            series_paint_spinner_box,
            paint_series_manager,
        });

        let buttons_c = Rc::clone(&tpm.buttons);
        tpm.mix_entry.name_entry.connect_changed(move |entry| {
            if entry.get_text_length() > 0 {
                buttons_c.update_condns(MaskedCondns {
                    condns: Self::SAV_HAS_NAME,
                    mask: Self::SAV_HAS_NAME,
                });
            } else {
                buttons_c.update_condns(MaskedCondns {
                    condns: 0,
                    mask: Self::SAV_HAS_NAME,
                });
            }
        });

        let tpm_c = Rc::clone(&tpm);
        tpm.paint_series_manager
            .connect_add_paint(move |paint| tpm_c.add_series_paint(&paint));

        let tpm_c = Rc::clone(&tpm);
        tpm.series_paint_spinner_box
            .connect_contributions_changed(move || tpm_c.contributions_changed());

        tpm
    }

    fn add_series_paint(&self, paint: &Rc<SeriesPaint<f64>>) {
        self.series_paint_spinner_box.add_paint(paint);
        self.hue_wheel.add_item(paint.coloured_shape());
    }

    fn contributions_changed(&self) {
        let mut colour_mixer = ColourMixer::new();
        for (rgb, parts) in self.series_paint_spinner_box.rgb_contributions() {
            colour_mixer.add(&rgb, parts);
        }
        if let Some(rgb) = colour_mixer.mixture() {
            self.mix_entry.set_mix_rgb(Some(&rgb));
            self.buttons.update_condns(MaskedCondns {
                condns: Self::SAV_HAS_COLOUR,
                mask: Self::SAV_HAS_COLOUR,
            });
        } else {
            self.mix_entry.set_mix_rgb(None);
            self.buttons.update_condns(MaskedCondns {
                condns: 0,
                mask: Self::SAV_HAS_COLOUR,
            });
        }
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.hue_wheel.set_target_rgb(rgb);
        self.mix_entry.set_target_rgb(rgb);
        self.paint_series_manager.set_target_rgb(rgb);
        if rgb.is_some() {
            self.buttons.update_condns(MaskedCondns {
                condns: Self::SAV_HAS_TARGET,
                mask: Self::HAS_TARGET_MASK,
            });
        } else {
            self.buttons.update_condns(MaskedCondns {
                condns: Self::SAV_NOT_HAS_TARGET,
                mask: Self::HAS_TARGET_MASK,
            });
        }
    }
}
