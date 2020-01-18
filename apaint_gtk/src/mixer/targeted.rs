// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    fs::File,
    path::Path,
    rc::Rc,
};

use gtk::prelude::*;

use cairo;

use pw_gix::{
    cairox::*,
    gtkx::paned::RememberPosition,
    sav_state::{ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled, SAV_NEXT_CONDN},
    wrapper::*,
};

use colour_math::ScalarAttribute;

use apaint::{
    characteristics::CharacteristicType,
    colour_mix::ColourMixer,
    hue_wheel::MakeColouredShape,
    mixtures::{MixedPaint, MixedPaintBuilder, MixingSession, Paint},
    series::SeriesPaint,
    BasicPaintIfce,
};

use crate::{
    attributes::ColourAttributeDisplayStack,
    colour::RGB,
    colour_edit::ColourEditor,
    hue_wheel::GtkHueWheel,
    icon_image::series_paint_image,
    list::{BasicPaintListViewSpec, ColouredItemListView, PaintListRow},
    mixer::component::{PartsSpinButtonBox, RcPartsSpinButtonBox},
    series::PaintSeriesManager,
    storage::{StorageManager, StorageManagerBuilder},
    window::PersistentWindowButtonBuilder,
};

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
        let id_label = gtk::LabelBuilder::new().label("MIX#???").build();
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

    pub fn target_rgb(&self) -> Option<RGB> {
        if let Some(rgb) = self.target_rgb.borrow().as_ref() {
            Some(*rgb)
        } else {
            None
        }
    }
}

#[derive(PWO, Wrapper)]
pub struct TargetedPaintMixer {
    vbox: gtk::Box,
    mixing_session: RefCell<MixingSession<f64>>,
    file_manager: Rc<StorageManager>,
    notes_entry: gtk::Entry,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    mix_entry: Rc<TargetedPaintEntry>,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    series_paint_spinner_box: Rc<PartsSpinButtonBox<SeriesPaint<f64>>>,
    paint_series_manager: Rc<PaintSeriesManager>,
    next_mix_id: Cell<u64>,
}

impl TargetedPaintMixer {
    const SAV_HAS_COLOUR: u64 = SAV_NEXT_CONDN << 0;
    const SAV_HAS_TARGET: u64 = SAV_NEXT_CONDN << 1;
    const SAV_NOT_HAS_TARGET: u64 = SAV_NEXT_CONDN << 2;
    const HAS_TARGET_MASK: u64 = Self::SAV_HAS_TARGET + Self::SAV_NOT_HAS_TARGET;
    const SAV_HAS_NAME: u64 = SAV_NEXT_CONDN << 3;

    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let file_manager = StorageManagerBuilder::new()
            .last_file_key("targeted_mixer::session")
            .tooltip_text(
                "reset",
                "Reset the mixer in preparation for a new mixing session",
            )
            .build();
        let notes_entry = gtk::EntryBuilder::new().build();
        let hue_wheel = GtkHueWheel::new(&[], attributes);
        let list_spec = BasicPaintListViewSpec::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(&list_spec, &[]);
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
            None,
        );
        buttons.update_condns(MaskedCondns {
            condns: Self::SAV_NOT_HAS_TARGET,
            mask: Self::HAS_TARGET_MASK,
        });
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let new_mix_btn = gtk::ButtonBuilder::new()
            .label("New")
            .tooltip_text("Start mixing a new colour.")
            .build();
        buttons.add_widget("new_mix", &new_mix_btn, Self::SAV_NOT_HAS_TARGET);
        button_box.pack_start(&new_mix_btn, true, true, 0);

        let accept_btn = gtk::ButtonBuilder::new()
            .label("Accept")
            .tooltip_text("Accept the current mixture and add it to the list of mixtures.")
            .build();
        buttons.add_widget(
            "accept",
            &accept_btn,
            Self::SAV_HAS_COLOUR + Self::SAV_HAS_TARGET + Self::SAV_HAS_NAME,
        );
        button_box.pack_start(&accept_btn, true, true, 0);

        let cancel_btn = gtk::ButtonBuilder::new()
            .label("Cancel")
            .tooltip_text("Cancel the current mixture.")
            .build();
        buttons.add_widget("cancel", &cancel_btn, Self::SAV_HAS_TARGET);
        button_box.pack_start(&cancel_btn, true, true, 0);

        let simplify_btn = gtk::ButtonBuilder::new()
            .label("Simplify Parts")
            .tooltip_text("Simplify the parts currently allocated to paints.")
            .build();
        buttons.add_widget("simplify", &simplify_btn, Self::SAV_HAS_COLOUR);
        button_box.pack_start(&simplify_btn, true, true, 0);

        let zero_parts_btn = gtk::ButtonBuilder::new()
            .label("Zero All Parts")
            .tooltip_text("Set the parts for all paints to zero.")
            .build();
        buttons.add_widget("zero_parts", &zero_parts_btn, Self::SAV_HAS_COLOUR);
        button_box.pack_start(&zero_parts_btn, true, true, 0);

        vbox.pack_start(&button_box, false, false, 0);
        vbox.pack_start(&series_paint_spinner_box.pwo(), false, false, 0);
        vbox.pack_start(&list_view.pwo(), true, true, 0);
        vbox.show_all();

        let tpm = Rc::new(Self {
            vbox,
            file_manager,
            notes_entry,
            mixing_session: RefCell::new(MixingSession::new()),
            hue_wheel,
            list_view,
            attributes: attributes.to_vec(),
            characteristics: characteristics.to_vec(),
            mix_entry,
            buttons,
            series_paint_spinner_box,
            paint_series_manager,
            next_mix_id: Cell::new(1),
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
        tpm.notes_entry.connect_changed(move |entry| {
            if let Some(text) = entry.get_text() {
                tpm_c.mixing_session.borrow_mut().set_notes(&text);
                tpm_c.update_session_needs_saving();
                tpm_c.update_session_is_saveable();
            }
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
        new_mix_btn.connect_clicked(move |_| tpm_c.ask_start_new_mixture());

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

        tpm
    }

    fn format_mix_id(&self) -> String {
        format!("MIX#{:03}", self.next_mix_id.get())
    }

    fn advance_mix_id(&self) {
        self.next_mix_id.set(self.next_mix_id.get() + 1);
    }

    fn add_series_paint(&self, paint: &Rc<SeriesPaint<f64>>) {
        self.series_paint_spinner_box.add_paint(paint);
        self.hue_wheel.add_item(paint.coloured_shape());
    }

    fn process_removal_request(&self, paint: &Rc<SeriesPaint<f64>>) {
        self.series_paint_spinner_box.remove_paint(paint);
        self.hue_wheel.remove_item(paint.id());
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

    fn ask_start_new_mixture(&self) {
        let tpe = TargetPaintEntry::new(&self.attributes);
        let dialog = self.new_dialog_with_buttons(
            Some("New Mixed Paint Target Colour"),
            gtk::DialogFlags::DESTROY_WITH_PARENT,
            CANCEL_OK_BUTTONS,
        );
        dialog
            .get_content_area()
            .pack_start(&tpe.pwo(), true, true, 0);
        if dialog.run() == gtk::ResponseType::Ok {
            let rgb = tpe.rgb();
            let name = tpe.name();
            let notes = tpe.notes();
            dialog.destroy();
            self.start_new_mixture(&name, &notes, &rgb);
        } else {
            dialog.destroy();
        }
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
            .update_session_is_saveable(self.mixing_session.borrow().notes().len() > 0);
    }

    fn write_to_file<Q: AsRef<Path>>(&self, path: Q) -> Result<Vec<u8>, apaint::Error> {
        let path: &Path = path.as_ref();
        let mut file = File::create(path)?;
        let new_digest = self.mixing_session.borrow_mut().write(&mut file)?;
        Ok(new_digest)
    }

    fn read_from_file<Q: AsRef<Path>>(&self, path: Q) -> Result<Vec<u8>, apaint::Error> {
        let path: &Path = path.as_ref();
        let mut file = File::open(path)?;
        let session = MixingSession::<f64>::read(&mut file, &self.paint_series_manager)?;
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
            self.hue_wheel.add_item(mixture.targeted_rgb_shape());
            self.list_view
                .add_row(&mixture.row(&self.attributes, &self.characteristics));
        }
        let digest = session.digest().expect("should work");
        *self.mixing_session.borrow_mut() = session;
        Ok(digest)
    }

    pub fn start_new_mixture(&self, name: &str, notes: &str, target_rgb: &RGB) {
        self.mix_entry.id_label.set_label(&self.format_mix_id());
        self.mix_entry.name_entry.set_text(name);
        self.mix_entry.notes_entry.set_text(notes);
        self.set_target_rgb(Some(target_rgb));
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
            self.file_manager.update_tool_needs_saving(true);
        } else {
            self.buttons.update_condns(MaskedCondns {
                condns: Self::SAV_NOT_HAS_TARGET,
                mask: Self::HAS_TARGET_MASK,
            });
            self.file_manager.update_tool_needs_saving(false);
        }
    }

    pub fn accept_current_mixture(&self) {
        let mix_id = self.format_mix_id();
        self.advance_mix_id();
        let mixed_paint = MixedPaintBuilder::<f64>::new(&mix_id)
            .name(&self.mix_entry.name_entry.get_text().unwrap_or("".into()))
            .notes(&self.mix_entry.notes_entry.get_text().unwrap_or("".into()))
            .targeted_rgb(
                &self
                    .mix_entry
                    .target_rgb()
                    .expect("should not be accepted without target"),
            )
            .series_paint_components(self.series_paint_spinner_box.paint_contributions())
            .build();
        self.hue_wheel.add_item(mixed_paint.coloured_shape());
        self.hue_wheel.add_item(mixed_paint.targeted_rgb_shape());
        self.list_view
            .add_row(&mixed_paint.row(&self.attributes, &self.characteristics));
        self.mix_entry.id_label.set_label("MIX#???");
        self.mix_entry.name_entry.set_text("");
        self.mix_entry.notes_entry.set_text("");
        self.set_target_rgb(None);
        self.series_paint_spinner_box.zero_all_parts();
        // TODO: handle case of duplicate mixed paint
        self.mixing_session.borrow_mut().add_mixture(&mixed_paint);
        self.update_session_needs_saving();
    }

    pub fn cancel_current_mixture(&self) {
        self.mix_entry.id_label.set_label("MIX#???");
        self.mix_entry.name_entry.set_text("");
        self.mix_entry.notes_entry.set_text("");
        self.set_target_rgb(None);
        self.series_paint_spinner_box.zero_all_parts();
    }

    pub fn full_reset(&self) -> Result<Vec<u8>, apaint::Error> {
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
}

#[derive(PWO)]
struct TargetPaintEntry {
    vbox: gtk::Box,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    colour_editor: Rc<ColourEditor>,
}

impl TargetPaintEntry {
    fn new(attributes: &[ScalarAttribute]) -> Self {
        // TODO: remember auto match on paste value
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let name_entry = gtk::EntryBuilder::new().hexpand(true).build();
        let notes_entry = gtk::EntryBuilder::new().hexpand(true).build();
        let colour_editor = ColourEditor::new(attributes, &[]);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&gtk::Label::new(Some(" Name:")), false, false, 0);
        hbox.pack_start(&name_entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&gtk::Label::new(Some("Notes:")), false, false, 0);
        hbox.pack_start(&notes_entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&colour_editor.pwo(), true, true, 0);
        vbox.show_all();
        Self {
            vbox,
            name_entry,
            notes_entry,
            colour_editor,
        }
    }

    fn name(&self) -> String {
        self.name_entry.get_text().unwrap_or("".into()).to_string()
    }

    fn notes(&self) -> String {
        self.notes_entry.get_text().unwrap_or("".into()).to_string()
    }

    fn rgb(&self) -> RGB {
        self.colour_editor.rgb()
    }
}
