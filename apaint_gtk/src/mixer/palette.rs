// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    fs::File,
    path::{Path, PathBuf},
    rc::Rc,
};

use pw_gix::{
    cairo, gdk, gdk_pixbuf,
    gtk::{self, prelude::*},
    gtkx::{menu::WrappedMenu, paned::RememberPosition},
    sav_state::{
        ChangedCondnsNotifier, ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled,
        SAV_HOVER_OK, SAV_NEXT_CONDN,
    },
    wrapper::*,
};

use colour_math::{beigui::hue_wheel::MakeColouredShape, ScalarAttribute, HCV};
use colour_math_cairo::{CairoSetColour, Point};

use colour_math_gtk::{
    attributes::{ColourAttributeDisplayStack, ColourAttributeDisplayStackBuilder},
    colour::GdkColour,
    hue_wheel::{GtkHueWheel, GtkHueWheelBuilder},
};

use apaint::{
    characteristics::CharacteristicType,
    colour_mix::ColourMixer,
    mixtures::{MixingSession, MixtureBuilder, Paint},
    series::SeriesPaint,
    BasicPaintIfce,
};

use crate::{
    colour::RGBConstants,
    icon_image::series_paint_image,
    list::{BasicPaintListViewSpec, ColouredItemListView, PaintListRow},
    mixer::{
        component::{PartsSpinButtonBox, RcPartsSpinButtonBox},
        display::{MixtureDisplayDialogManager, MixtureDisplayDialogManagerBuilder},
    },
    series::{PaintSeriesManager, PaintSeriesManagerBuilder},
    storage::{StorageManager, StorageManagerBuilder},
    window::PersistentWindowButtonBuilder,
};

struct Sample {
    pixbuf: gdk_pixbuf::Pixbuf,
    position: Point,
}

#[derive(PWO, Wrapper)]
pub struct PalettePaintEntry {
    vbox: gtk::Box,
    id_label: gtk::Label,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    cads: Rc<ColourAttributeDisplayStack>,
    drawing_area: gtk::DrawingArea,
    mix_colour: RefCell<Option<HCV>>,
    samples: RefCell<Vec<Sample>>,
    popup_menu: WrappedMenu,
    popup_menu_posn: Cell<Point>,
}

impl PalettePaintEntry {
    pub fn new(attributes: &[ScalarAttribute]) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let id_label = gtk::LabelBuilder::new().label("MIX#???").build();
        let name_entry = gtk::EntryBuilder::new().build();
        let notes_entry = gtk::EntryBuilder::new().build();
        let cads = ColourAttributeDisplayStackBuilder::new()
            .attributes(attributes)
            .build();
        let drawing_area = gtk::DrawingAreaBuilder::new()
            .events(gdk::EventMask::BUTTON_PRESS_MASK)
            .height_request(100)
            .build();
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
            mix_colour: RefCell::new(None),
            samples: RefCell::new(Vec::new()),
            popup_menu: WrappedMenu::new(&[]),
            popup_menu_posn: Cell::new((0.0, 0.0).into()),
        });

        // POPUP
        let tpe_c = Rc::clone(&tpe);
        tpe.popup_menu
            .append_item(
                "paste",
                "Paste Sample",
                "Paste image sample from the clipboard at this position",
            )
            .connect_activate(move |_| {
                let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                if let Some(pixbuf) = cbd.wait_for_image() {
                    let sample = Sample {
                        pixbuf,
                        position: tpe_c.popup_menu_posn.get(),
                    };
                    tpe_c.samples.borrow_mut().push(sample);
                    tpe_c.drawing_area.queue_draw();
                } else {
                    tpe_c.inform_user("No image data on clipboard.", None);
                }
            });
        let tpe_c = Rc::clone(&tpe);
        tpe.popup_menu
            .append_item(
                "remove",
                "Remove Sample(s)",
                "Remove all image samples from the sample area",
            )
            .connect_activate(move |_| {
                tpe_c.samples.borrow_mut().clear();
                tpe_c.drawing_area.queue_draw();
            });
        let tpe_c = Rc::clone(&tpe);
        tpe.drawing_area
            .connect_button_press_event(move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress && event.get_button() == 3
                {
                    let position = Point::from(event.get_position());
                    let n_samples = tpe_c.samples.borrow().len();
                    let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                    tpe_c
                        .popup_menu
                        .set_sensitivities(cbd.wait_is_image_available(), &["paste"]);
                    tpe_c
                        .popup_menu
                        .set_sensitivities(n_samples > 0, &["remove"]);
                    tpe_c.popup_menu_posn.set(position);
                    tpe_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                }
                Inhibit(false)
            });

        let tpe_c = Rc::clone(&tpe);
        tpe.drawing_area.connect_draw(move |da, ctxt| {
            tpe_c.draw(da, ctxt);
            Inhibit(false)
        });

        tpe
    }

    pub fn draw(&self, _drawing_area: &gtk::DrawingArea, cairo_context: &cairo::Context) {
        if let Some(ref colour) = *self.mix_colour.borrow() {
            cairo_context.set_source_colour(colour);
        } else {
            cairo_context.set_source_colour(&HCV::BLACK);
        };
        cairo_context.paint();
        for sample in self.samples.borrow().iter() {
            let buffer = sample
                .pixbuf
                .save_to_bufferv("png", &[])
                .expect("pixbuf to png error");
            let mut reader = std::io::Cursor::new(buffer);
            let surface = cairo::ImageSurface::create_from_png(&mut reader).unwrap();
            cairo_context.set_source_surface(&surface, sample.position.x, sample.position.y);
            cairo_context.paint();
        }
    }

    pub fn set_mix_colour(&self, colour: Option<&impl GdkColour>) {
        if let Some(colour) = colour {
            *self.mix_colour.borrow_mut() = Some(colour.hcv());
            self.cads.set_colour(Some(colour));
        } else {
            *self.mix_colour.borrow_mut() = None;
            self.cads.set_colour(Option::<&HCV>::None);
        }
        self.drawing_area.queue_draw()
    }

    pub fn delete_samples(&self) {
        self.samples.borrow_mut().clear();
    }
}

#[derive(PWO, Wrapper)]
pub struct PalettePaintMixer {
    vbox: gtk::Box,
    mixing_session: RefCell<MixingSession>,
    file_manager: Rc<StorageManager>,
    notes_entry: gtk::Entry,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    mix_entry: Rc<PalettePaintEntry>,
    series_paint_spinner_box: Rc<PartsSpinButtonBox<SeriesPaint>>,
    change_notifier: Rc<ChangedCondnsNotifier>,
    paint_series_manager: Rc<PaintSeriesManager>,
    next_mix_id: Cell<u64>,
    display_dialog_manager: RefCell<MixtureDisplayDialogManager<gtk::Box>>,
}

impl PalettePaintMixer {
    const SAV_HAS_COLOUR: u64 = SAV_NEXT_CONDN;
    const SAV_HAS_TARGET: u64 = SAV_NEXT_CONDN << 1;
    pub const SAV_NOT_HAS_TARGET: u64 = SAV_NEXT_CONDN << 2;
    const SAV_HAS_NAME: u64 = SAV_NEXT_CONDN << 3;

    fn format_mix_id(&self) -> String {
        format!("MIX#{:03}", self.next_mix_id.get())
    }

    fn advance_mix_id(&self) {
        self.next_mix_id.set(self.next_mix_id.get() + 1);
    }

    fn add_series_paint(&self, paint: &Rc<SeriesPaint>) {
        self.series_paint_spinner_box.add_paint(paint);
        self.hue_wheel.add_item(paint.coloured_shape());
    }

    fn process_removal_request(&self, paint: &Rc<SeriesPaint>) {
        self.series_paint_spinner_box.remove_paint(paint);
        self.hue_wheel.remove_item(paint.id());
    }

    fn contributions_changed(&self) {
        let mut colour_mixer = ColourMixer::new();
        for (rgb, parts) in self.series_paint_spinner_box.rgb_contributions() {
            colour_mixer.add(&rgb, parts);
        }
        let mut condns = MaskedCondns {
            condns: 0,
            mask: Self::SAV_HAS_COLOUR,
        };
        if let Some(colour) = colour_mixer.mixture() {
            self.mix_entry.set_mix_colour(Some(&colour));
            condns.condns = Self::SAV_HAS_COLOUR;
        } else {
            self.mix_entry.set_mix_colour(Option::<&HCV>::None);
        }
        self.change_notifier.notify_changed_condns(condns);
    }

    fn update_session_needs_saving(&self) {
        let digest = self
            .mixing_session
            .borrow()
            .digest()
            .expect("unrecoverable");
        self.file_manager.update_session_needs_saving(&digest);
    }

    fn update_session_is_saveable(&self) {
        self.file_manager
            .update_session_is_saveable(!self.mixing_session.borrow().notes().is_empty());
    }

    fn write_to_file<Q: AsRef<Path>>(&self, path: Q) -> apaint::Result<Vec<u8>> {
        let path: &Path = path.as_ref();
        let mut file = File::create(path)?;
        let new_digest = self.mixing_session.borrow_mut().write(&mut file)?;
        Ok(new_digest)
    }

    fn read_from_file<Q: AsRef<Path>>(&self, path: Q) -> apaint::Result<Vec<u8>> {
        let path: &Path = path.as_ref();
        let mut file = File::open(path)?;
        let session = MixingSession::read(&mut file, &self.paint_series_manager)?;
        // TODO: completely clear the mixer
        self.notes_entry.set_text(session.notes());
        for mixture in session.mixtures() {
            for (paint, _) in mixture.components() {
                match paint {
                    Paint::Series(series_paint) => {
                        self.add_series_paint(series_paint);
                    }
                    Paint::Mixed(_mixed_paint) => {
                        // TODO: add mixed paints to spinners
                    }
                }
            }
            self.hue_wheel.add_item(mixture.coloured_shape());
            self.list_view
                .add_row(&mixture.row(&self.attributes, &self.characteristics));
        }
        let digest = session.digest().expect("should work");
        *self.mixing_session.borrow_mut() = session;
        Ok(digest)
    }

    // TODO: review visibility of palette mixer methods
    pub fn start_new_mixture(&self) {
        self.mix_entry.id_label.set_label(&self.format_mix_id());
        self.mix_entry.name_entry.set_text("");
        self.mix_entry.notes_entry.set_text("");
    }

    pub fn accept_current_mixture(&self) {
        let mix_id = self.format_mix_id();
        self.advance_mix_id();
        let mixed_paint = MixtureBuilder::new(&mix_id)
            .name(&self.mix_entry.name_entry.get_text())
            .notes(&self.mix_entry.notes_entry.get_text())
            .series_paint_components(self.series_paint_spinner_box.paint_contributions())
            .build();
        self.hue_wheel.add_item(mixed_paint.coloured_shape());
        self.list_view
            .add_row(&mixed_paint.row(&self.attributes, &self.characteristics));
        self.mix_entry.id_label.set_label("MIX#???");
        self.mix_entry.name_entry.set_text("");
        self.mix_entry.notes_entry.set_text("");
        self.series_paint_spinner_box.zero_all_parts();
        // TODO: handle case of duplicate mixed paint
        self.mixing_session.borrow_mut().add_mixture(&mixed_paint);
        self.update_session_needs_saving();
    }

    pub fn cancel_current_mixture(&self) {
        self.mix_entry.id_label.set_label("MIX#???");
        self.mix_entry.name_entry.set_text("");
        self.mix_entry.notes_entry.set_text("");
        self.series_paint_spinner_box.zero_all_parts();
    }

    pub fn full_reset(&self) -> apaint::Result<Vec<u8>> {
        self.mix_entry.delete_samples();
        self.notes_entry.set_text("");
        self.cancel_current_mixture();
        *self.mixing_session.borrow_mut() = MixingSession::new();
        let digest = self.mixing_session.borrow().digest().expect("should work");
        Ok(digest)
    }

    pub fn simplify_current_parts(&self) {
        let gcd = self.series_paint_spinner_box.parts_gcd();
        self.series_paint_spinner_box.div_all_parts_by(gcd);
    }

    pub fn zero_all_parts(&self) {
        self.series_paint_spinner_box.zero_all_parts();
    }

    pub fn needs_saving(&self) -> bool {
        self.file_manager.needs_saving()
    }
}

#[derive(Default)]
pub struct PalettePaintMixerBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    config_dir_path: Option<PathBuf>,
}

impl PalettePaintMixerBuilder {
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

    pub fn config_dir_path(&mut self, path: &Path) -> &mut Self {
        self.config_dir_path = Some(path.to_path_buf());
        self
    }

    pub fn build(&self) -> Rc<PalettePaintMixer> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let file_manager = StorageManagerBuilder::new()
            .last_file_key("palette_mixer::session")
            .tooltip_text(
                "reset",
                "Reset the mixer in preparation for a new mixing session",
            )
            .build();
        let change_notifier = ChangedCondnsNotifier::new(PalettePaintMixer::SAV_NOT_HAS_TARGET);
        let notes_entry = gtk::EntryBuilder::new().build();
        let hue_wheel = GtkHueWheelBuilder::new()
            .attributes(&self.attributes)
            .build();
        let list_spec = BasicPaintListViewSpec::new(&self.attributes, &self.characteristics);
        let list_view = ColouredItemListView::new(
            &list_spec,
            &[(
                "info",
                (
                    "Paint Information",
                    None,
                    Some("Display information for the indicated paint"),
                )
                    .into(),
                SAV_HOVER_OK,
            )],
        );
        let mix_entry = PalettePaintEntry::new(&self.attributes);
        let series_paint_spinner_box = PartsSpinButtonBox::<SeriesPaint>::new("Paints", 4, true);

        let display_dialog_manager = MixtureDisplayDialogManagerBuilder::new(&vbox)
            .attributes(&self.attributes)
            .characteristics(&self.characteristics)
            .build();

        let mut builder = PaintSeriesManagerBuilder::new();
        builder
            .attributes(&self.attributes)
            .characteristics(&self.characteristics)
            .change_notifier(&change_notifier);
        if let Some(ref config_dir_path) = self.config_dir_path {
            builder.loaded_files_data_path(&config_dir_path.join("paint_series_files"));
        }
        let paint_series_manager = builder.build();
        let persistent_window_btn = PersistentWindowButtonBuilder::new()
            .icon(&series_paint_image(24))
            .window_child(&paint_series_manager.pwo())
            .window_title("Paint Series Manager")
            .window_geometry(Some("paint_series_manager"), (300, 200))
            .build();
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        button_box.pack_start(&persistent_window_btn.pwo(), false, false, 0);

        button_box.pack_start(&file_manager.pwo(), true, true, 0);
        vbox.pack_start(&button_box, false, false, 0);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&gtk::Label::new(Some("Notes:")), false, false, 0);
        hbox.pack_start(&notes_entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.add1(&hue_wheel.pwo());
        paned.add2(&mix_entry.pwo());
        paned.set_position_from_recollections("basic paint factory h paned position", 200);
        vbox.pack_start(&paned, true, true, 0);
        let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
            WidgetStatesControlled::Sensitivity,
            None,
            Some(&change_notifier),
        );
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let new_mix_btn = gtk::ButtonBuilder::new()
            .label("New")
            .tooltip_text("Start mixing a new colour.")
            .build();
        buttons.add_widget(
            "new_mix",
            &new_mix_btn,
            PalettePaintMixer::SAV_NOT_HAS_TARGET,
        );
        button_box.pack_start(&new_mix_btn, true, true, 0);

        let accept_btn = gtk::ButtonBuilder::new()
            .label("Accept")
            .tooltip_text("Accept the current mixture and add it to the list of mixtures.")
            .build();
        buttons.add_widget(
            "accept",
            &accept_btn,
            PalettePaintMixer::SAV_HAS_COLOUR
                + PalettePaintMixer::SAV_HAS_TARGET
                + PalettePaintMixer::SAV_HAS_NAME,
        );
        button_box.pack_start(&accept_btn, true, true, 0);

        let cancel_btn = gtk::ButtonBuilder::new()
            .label("Cancel")
            .tooltip_text("Cancel the current mixture.")
            .build();
        buttons.add_widget("cancel", &cancel_btn, PalettePaintMixer::SAV_HAS_TARGET);
        button_box.pack_start(&cancel_btn, true, true, 0);

        let simplify_btn = gtk::ButtonBuilder::new()
            .label("Simplify Parts")
            .tooltip_text("Simplify the parts currently allocated to paints.")
            .build();
        buttons.add_widget("simplify", &simplify_btn, PalettePaintMixer::SAV_HAS_COLOUR);
        button_box.pack_start(&simplify_btn, true, true, 0);

        let zero_parts_btn = gtk::ButtonBuilder::new()
            .label("Zero All Parts")
            .tooltip_text("Set the parts for all paints to zero.")
            .build();
        buttons.add_widget(
            "zero_parts",
            &zero_parts_btn,
            PalettePaintMixer::SAV_HAS_COLOUR,
        );
        button_box.pack_start(&zero_parts_btn, true, true, 0);

        vbox.pack_start(&button_box, false, false, 0);
        vbox.pack_start(&series_paint_spinner_box.pwo(), false, false, 0);
        vbox.pack_start(&list_view.pwo(), true, true, 0);
        vbox.show_all();

        let tpm = Rc::new(PalettePaintMixer {
            vbox,
            file_manager,
            notes_entry,
            mixing_session: RefCell::new(MixingSession::new()),
            hue_wheel,
            list_view,
            attributes: self.attributes.clone(),
            characteristics: self.characteristics.clone(),
            mix_entry,
            series_paint_spinner_box,
            change_notifier,
            paint_series_manager,
            next_mix_id: Cell::new(1),
            display_dialog_manager: RefCell::new(display_dialog_manager),
        });

        let change_notifier_c = Rc::clone(&tpm.change_notifier);
        tpm.mix_entry.name_entry.connect_changed(move |entry| {
            let mut condns = MaskedCondns {
                condns: 0,
                mask: PalettePaintMixer::SAV_HAS_NAME,
            };
            if entry.get_text_length() > 0 {
                condns.condns = PalettePaintMixer::SAV_HAS_NAME;
            };
            change_notifier_c.notify_changed_condns(condns);
        });

        let tpm_c = Rc::clone(&tpm);
        tpm.notes_entry.connect_changed(move |entry| {
            let text = entry.get_text();
            tpm_c.mixing_session.borrow_mut().set_notes(&text);
            tpm_c.update_session_needs_saving();
            tpm_c.update_session_is_saveable();
        });

        let tpm_c = Rc::clone(&tpm);
        tpm.paint_series_manager
            .connect_add_paint(move |paint| tpm_c.add_series_paint(&paint));

        let tpm_c = Rc::clone(&tpm);
        tpm.series_paint_spinner_box
            .connect_contributions_changed(move || tpm_c.contributions_changed());

        let tpm_c = Rc::clone(&tpm);
        tpm.series_paint_spinner_box
            .connect_removal_requested(move |p| tpm_c.process_removal_request(p));

        let tpm_c = Rc::clone(&tpm);
        new_mix_btn.connect_clicked(move |_| tpm_c.start_new_mixture());

        let tpm_c = Rc::clone(&tpm);
        accept_btn.connect_clicked(move |_| tpm_c.accept_current_mixture());

        let tpm_c = Rc::clone(&tpm);
        cancel_btn.connect_clicked(move |_| tpm_c.cancel_current_mixture());

        let tpm_c = Rc::clone(&tpm);
        simplify_btn.connect_clicked(move |_| tpm_c.simplify_current_parts());

        let tpm_c = Rc::clone(&tpm);
        zero_parts_btn.connect_clicked(move |_| tpm_c.zero_all_parts());

        // FILE MANAGEMENT
        let tpm_c = Rc::clone(&tpm);
        tpm.file_manager
            .connect_save(move |path| tpm_c.write_to_file(path));

        let tpm_c = Rc::clone(&tpm);
        tpm.file_manager
            .connect_load(move |path| tpm_c.read_from_file(path));

        let tpm_c = Rc::clone(&tpm);
        tpm.file_manager.connect_reset(move || tpm_c.full_reset());

        let tpm_c = Rc::clone(&tpm);
        tpm.list_view.connect_popup_menu_item("info", move |id| {
            let mixing_session = tpm_c.mixing_session.borrow();
            let mixture = mixing_session.mixture(id).expect("programm error");
            tpm_c
                .display_dialog_manager
                .borrow_mut()
                .display_mixture(mixture);
        });

        tpm
    }
}
