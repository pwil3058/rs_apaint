// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;
use pw_gix::wrapper::*;

use pw_gix::gtkx::coloured::Colourable;
use pw_gix::gtkx::paned::RememberPosition;
use pw_gix::sav_state::{
    ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled, SAV_HOVER_OK, SAV_NEXT_CONDN,
};

use colour_math::{ColourInterface, ScalarAttribute};

use apaint::{
    characteristics::CharacteristicType,
    hue_wheel::MakeColouredShape,
    spec::{BasicPaintSeriesSpec, BasicPaintSpec},
    BasicPaintIfce,
};

use apaint_gtk_boilerplate::{Wrapper, PWO};

use crate::colour::RGB;
use crate::hue_wheel::GtkHueWheel;
use crate::icon_image;
use crate::icon_image::{needs_save_not_ready_image, needs_save_ready_image, up_to_date_image};
use crate::list::{BasicPaintListViewSpec, ColouredItemListView, PaintListRow};
use crate::managed_menu::MenuItemSpec;
use crate::spec_edit::BasicPaintSpecEditor;

#[derive(PWO)]
struct FactoryFileManager {
    hbox: gtk::Box,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    file_name_label: gtk::Label,
    file_status_btn: gtk::Button,
    current_file_path: RefCell<Option<PathBuf>>,
}

impl FactoryFileManager {
    const SAV_HAS_CURRENT_FILE: u64 = SAV_NEXT_CONDN << 0;
    const SAV_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 1;
    const SAV_EDITOR_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 2;
    const SAV_SERIES_NEEDS_SAVING: u64 = SAV_NEXT_CONDN << 3;
    const SAV_SERIES_IS_SAVEABLE: u64 = SAV_NEXT_CONDN << 4;

    const BTN_IMAGE_SIZE: i32 = 24;

    fn new() -> Self {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
            WidgetStatesControlled::Sensitivity,
            None,
            None,
        );

        let new_colln_btn = gtk::ButtonBuilder::new()
            .tooltip_text("Clear the editor in preparation for creating a new collection")
            .build();
        // TODO: change setting of image when ButtonBuilder interface is fixed.
        new_colln_btn.set_image(Some(&icon_image::colln_new_image(Self::BTN_IMAGE_SIZE)));
        buttons.add_widget("new_colln", &new_colln_btn, 0);
        hbox.pack_start(&new_colln_btn, false, false, 0);

        let load_colln_btn = gtk::ButtonBuilder::new()
            .tooltip_text("Load a paint collection from a file for editing.")
            .build();
        // TODO: change setting of image when ButtonBuilder interface is fixed.
        load_colln_btn.set_image(Some(&icon_image::colln_load_image(Self::BTN_IMAGE_SIZE)));
        buttons.add_widget("load_colln", &load_colln_btn, 0);
        hbox.pack_start(&load_colln_btn, false, false, 0);

        let save_colln_btn = gtk::ButtonBuilder::new()
            .tooltip_text("Save the current editor content to the current file.")
            .build();
        // TODO: change setting of image when ButtonBuilder interface is fixed.
        save_colln_btn.set_image(Some(&icon_image::colln_save_image(Self::BTN_IMAGE_SIZE)));
        buttons.add_widget(
            "save_colln",
            &save_colln_btn,
            Self::SAV_HAS_CURRENT_FILE + Self::SAV_SERIES_IS_SAVEABLE,
        );
        hbox.pack_start(&save_colln_btn, false, false, 0);

        let save_as_colln_btn = gtk::ButtonBuilder::new()
            .tooltip_text("Save the current editor content to a nominated file.")
            .build();
        // TODO: change setting of image when ButtonBuilder interface is fixed.
        save_as_colln_btn.set_image(Some(&icon_image::colln_save_as_image(Self::BTN_IMAGE_SIZE)));
        buttons.add_widget(
            "save_as_colln",
            &save_as_colln_btn,
            Self::SAV_SERIES_IS_SAVEABLE,
        );
        hbox.pack_start(&save_as_colln_btn, false, false, 0);

        hbox.pack_start(&gtk::Label::new(Some("Current File:")), false, false, 1);
        let file_name_label = gtk::LabelBuilder::new()
            .justify(gtk::Justification::Left)
            .xalign(0.01)
            .build();
        file_name_label.set_widget_colour_rgb(RGB::WHITE);
        hbox.pack_start(&file_name_label, true, true, 1);

        let file_status_btn = gtk::ButtonBuilder::new().sensitive(false).build();
        file_status_btn.set_image(Some(&up_to_date_image(Self::BTN_IMAGE_SIZE)));
        hbox.pack_start(&file_status_btn, false, false, 1);
        buttons.add_widget(
            "file_status",
            &file_status_btn,
            Self::SAV_SERIES_IS_SAVEABLE + Self::SAV_SERIES_NEEDS_SAVING,
        );

        hbox.show_all();

        Self {
            hbox,
            buttons,
            file_name_label,
            file_status_btn,
            current_file_path: RefCell::new(None),
        }
    }

    fn set_current_file_path<Q: AsRef<Path>>(&self, path: Option<Q>) {
        let mut condns: u64 = 0;
        let mask: u64 = Self::SAV_HAS_CURRENT_FILE;
        if let Some(path) = path {
            let path: PathBuf = path.as_ref().to_path_buf();
            self.file_name_label.set_label(&path.to_string_lossy());
            *self.current_file_path.borrow_mut() = Some(path);
            condns = Self::SAV_HAS_CURRENT_FILE;
        } else {
            *self.current_file_path.borrow_mut() = None;
            self.file_name_label.set_label("")
        }
        self.buttons.update_condns(MaskedCondns { condns, mask });
    }

    fn update_file_status_button(&self) {
        let current_condns = self.buttons.current_condns();
        if current_condns & Self::SAV_SERIES_NEEDS_SAVING != 0 {
            if current_condns & Self::SAV_SERIES_IS_SAVEABLE != 0 {
                self.file_status_btn
                    .set_image(Some(&needs_save_ready_image(24)));
                self.file_status_btn.set_tooltip_text(Some(
                    "File Status: Needs Save (Ready)\nClick to save data to file",
                ));
            } else {
                self.file_status_btn
                    .set_image(Some(&needs_save_not_ready_image(24)));
                self.file_status_btn
                    .set_tooltip_text(Some("File Status: Needs Save (NOT Ready)"));
            }
        } else {
            self.file_status_btn.set_image(Some(&up_to_date_image(24)));
            self.file_status_btn
                .set_tooltip_text(Some("File Status: Up To Date"));
        }
    }
}

#[derive(PWO, Wrapper)]
pub struct BasicPaintFactory {
    vbox: gtk::Box,
    file_manager: FactoryFileManager,
    paint_editor: Rc<BasicPaintSpecEditor>,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    paint_series: RefCell<BasicPaintSeriesSpec<f64>>,
    saved_series_digest: RefCell<Vec<u8>>,
    proprietor_entry: gtk::Entry,
    series_name_entry: gtk::Entry,
}

impl BasicPaintFactory {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let menu_items: &[MenuItemSpec] = &[
            (
                "edit",
                "Edit",
                None,
                "Edit the indicated paint",
                SAV_HOVER_OK,
            )
                .into(),
            (
                "remove",
                "Remove",
                None,
                "Remove the indicated paint from the series.",
                SAV_HOVER_OK,
            )
                .into(),
        ];
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let grid = gtk::GridBuilder::new().hexpand(true).build();
        vbox.pack_start(&grid, false, false, 0);
        let label = gtk::LabelBuilder::new()
            .label("Series Name:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 0, 1, 1);
        let series_name_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&series_name_entry, 1, 0, 1, 1);
        let label = gtk::LabelBuilder::new()
            .label("Proprietor:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 1, 1, 1);
        let proprietor_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&proprietor_entry, 1, 1, 1, 1);
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        let paint_editor = BasicPaintSpecEditor::new(attributes, characteristics);
        let hue_wheel = GtkHueWheel::new(menu_items, attributes);
        let paint_list_spec = BasicPaintListViewSpec::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(&paint_list_spec, menu_items);
        let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
        scrolled_window.add(&list_view.pwo());
        let notebook = gtk::NotebookBuilder::new().build();
        notebook.add(&scrolled_window);
        notebook.set_tab_label_text(&scrolled_window, "Paint List");
        notebook.add(&hue_wheel.pwo());
        notebook.set_tab_label_text(&hue_wheel.pwo(), "Hue/Attribute Wheel");
        vbox.pack_start(&notebook, true, true, 0);
        paned.add1(&vbox);
        paned.add2(&paint_editor.pwo());
        paned.set_position_from_recollections("basic paint factory h paned position", 200);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let file_manager = FactoryFileManager::new();
        vbox.pack_start(&file_manager.pwo(), false, false, 0);
        vbox.pack_start(&paned, true, true, 0);
        let bpf = Rc::new(Self {
            vbox,
            file_manager,
            paint_editor,
            hue_wheel,
            list_view,
            attributes: attributes.to_vec(),
            characteristics: characteristics.to_vec(),
            paint_series: RefCell::new(BasicPaintSeriesSpec::default()),
            saved_series_digest: RefCell::new(vec![]),
            proprietor_entry,
            series_name_entry,
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.paint_editor
            .connect_add_action(move |spec| bpf_c.add_paint(spec));

        let bpf_c = Rc::clone(&bpf);
        bpf.paint_editor
            .connect_accept_action(move |id, spec| bpf_c.replace_paint(id, spec));

        let bpf_c = Rc::clone(&bpf);
        bpf.paint_editor
            .connect_changed(move |_| bpf_c.update_editor_needs_saving());

        let bpf_c = Rc::clone(&bpf);
        bpf.hue_wheel
            .connect_popup_menu_item("edit", move |id| bpf_c.edit_paint(id));

        let bpf_c = Rc::clone(&bpf);
        bpf.list_view
            .connect_popup_menu_item("edit", move |id| bpf_c.edit_paint(id));

        let bpf_c = Rc::clone(&bpf);
        bpf.hue_wheel
            .connect_popup_menu_item("remove", move |id| bpf_c.remove_paint(id));

        let bpf_c = Rc::clone(&bpf);
        bpf.list_view
            .connect_popup_menu_item("remove", move |id| bpf_c.remove_paint(id));

        let bpf_c = Rc::clone(&bpf);
        bpf.proprietor_entry.connect_changed(move |entry| {
            if let Some(text) = entry.get_text() {
                bpf_c.paint_series.borrow_mut().set_proprietor(&text);
                bpf_c.update_saveability();
                bpf_c.update_series_needs_saving();
            }
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.series_name_entry.connect_changed(move |entry| {
            if let Some(text) = entry.get_text() {
                bpf_c.paint_series.borrow_mut().set_series_name(&text);
                bpf_c.update_saveability();
                bpf_c.update_series_needs_saving();
            }
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager
            .buttons
            .get_widget("new_colln")
            .unwrap()
            .connect_clicked(move |_| bpf_c.reset());

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager
            .buttons
            .get_widget("load_colln")
            .unwrap()
            .connect_clicked(move |_| bpf_c.load());

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager
            .buttons
            .get_widget("save_colln")
            .unwrap()
            .connect_clicked(move |_| bpf_c.save());

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager
            .buttons
            .get_widget("save_as_colln")
            .unwrap()
            .connect_clicked(move |_| bpf_c.save_as());

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager.file_status_btn.connect_clicked(move |_| {
            if bpf_c.file_manager.current_file_path.borrow().is_some() {
                bpf_c.save()
            } else {
                bpf_c.save_as()
            }
        });

        bpf
    }

    fn update_saveability(&self) {
        let mut condns: u64 = 0;
        let mask: u64 =
            FactoryFileManager::SAV_IS_SAVEABLE + FactoryFileManager::SAV_SERIES_IS_SAVEABLE;
        let series = self.paint_series.borrow();
        let series_id = series.series_id();
        if series_id.proprietor().len() > 0 && series_id.series_name().len() > 0 {
            condns += FactoryFileManager::SAV_SERIES_IS_SAVEABLE;
            if self.file_manager.buttons.current_condns()
                & FactoryFileManager::SAV_EDITOR_NEEDS_SAVING
                == 0
            {
                condns += FactoryFileManager::SAV_IS_SAVEABLE;
            }
        }
        self.file_manager
            .buttons
            .update_condns(MaskedCondns { condns, mask });
        self.file_manager.update_file_status_button();
    }

    fn update_series_needs_saving(&self) {
        let mut condns: u64 = 0;
        let mask = FactoryFileManager::SAV_SERIES_NEEDS_SAVING;
        let digest = self.paint_series.borrow().digest().expect("unrecoverable");
        if digest != *self.saved_series_digest.borrow() {
            condns = FactoryFileManager::SAV_SERIES_NEEDS_SAVING;
        };
        self.file_manager
            .buttons
            .update_condns(MaskedCondns { condns, mask });
        self.file_manager.update_file_status_button();
    }

    fn update_editor_needs_saving(&self) {
        let mut condns: u64 = 0;
        let mask =
            FactoryFileManager::SAV_EDITOR_NEEDS_SAVING + FactoryFileManager::SAV_IS_SAVEABLE;
        if self.paint_editor.has_unsaved_changes() {
            condns += FactoryFileManager::SAV_EDITOR_NEEDS_SAVING;
        } else if self.file_manager.buttons.current_condns()
            & FactoryFileManager::SAV_SERIES_IS_SAVEABLE
            != 0
        {
            condns += FactoryFileManager::SAV_IS_SAVEABLE;
        }
        self.file_manager
            .buttons
            .update_condns(MaskedCondns { condns, mask });
        self.file_manager.update_file_status_button();
    }

    fn do_add_paint_work(&self, paint_spec: &BasicPaintSpec<f64>) {
        if let Some(old_paint) = self.paint_series.borrow_mut().add(paint_spec) {
            self.hue_wheel.remove_item(old_paint.id());
            self.list_view.remove_row(old_paint.id());
        }
        self.hue_wheel.add_item(paint_spec.coloured_shape());
        let row = paint_spec.row(&self.attributes, &self.characteristics);
        self.list_view.add_row(&row);
    }

    fn do_remove_paint_work(&self, id: &str) -> Result<(), apaint::Error> {
        self.paint_series.borrow_mut().remove(id)?;
        self.hue_wheel.remove_item(id);
        self.list_view.remove_row(id);
        Ok(())
    }

    fn add_paint(&self, paint_spec: &BasicPaintSpec<f64>) {
        self.do_add_paint_work(paint_spec);
        self.update_series_needs_saving();
    }

    fn remove_paint(&self, id: &str) {
        let question = format!("Confirm remove '{}'?", id);
        if self.ask_confirm_action(&question, None) {
            self.do_remove_paint_work(id).expect("should be successful");
            self.paint_editor.un_edit(id);
            self.update_series_needs_saving();
        }
    }

    fn replace_paint(&self, id: &str, paint_spec: &BasicPaintSpec<f64>) {
        // should not be called if paint has been removed after being chosen for edit
        self.do_remove_paint_work(id)
            .expect("should not be called if paint has been removed");
        self.do_add_paint_work(paint_spec);
        self.update_series_needs_saving();
    }

    fn edit_paint(&self, id: &str) {
        if self.paint_editor.has_unsaved_changes() {
            // NB: can't offer "save" as an option as it could change indicated paint
            let buttons = &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Continue Discarding Changes", gtk::ResponseType::Accept),
            ];
            if self.ask_question("Current paint has unsaved changes!", None, buttons)
                == gtk::ResponseType::Cancel
            {
                return;
            }
        }
        let paint_series = self.paint_series.borrow();
        let paint = paint_series.find(id).expect("should be there");
        let mut spec = BasicPaintSpec::<f64>::new(paint.rgb(), paint.id());
        if let Some(name) = paint.name() {
            spec.name = name.to_string();
        }
        if let Some(notes) = paint.notes() {
            spec.notes = notes.to_string();
        }
        spec.finish = paint.finish();
        spec.permanence = paint.permanence();
        spec.transparency = paint.transparency();
        spec.fluorescence = paint.fluorescence();
        spec.metallicness = paint.metallicness();
        self.paint_editor.edit(&spec);
    }

    fn write_to_file<Q: AsRef<Path>>(&self, path: Q) -> Result<(), apaint::Error> {
        let path: &Path = path.as_ref();
        let mut file = File::create(path)?;
        let new_digest = self.paint_series.borrow_mut().write(&mut file)?;
        self.file_manager.set_current_file_path(Some(path));
        *self.saved_series_digest.borrow_mut() = new_digest;
        self.update_series_needs_saving();
        Ok(())
    }

    fn save(&self) {
        let path = self
            .file_manager
            .current_file_path
            .borrow()
            .clone()
            .expect("programming error: save() should not have been called.");
        if let Err(err) = self.write_to_file(path) {
            self.report_error("Problem saving file", &err);
        }
    }

    fn save_as(&self) {
        // TODO: use last dir data option
        if let Some(path) = self.ask_file_path(Some("Save as: "), None, false) {
            if let Err(err) = self.write_to_file(path) {
                self.report_error("Problem saving file", &err);
            }
        }
    }

    fn ok_to_reset(&self) -> bool {
        let status = self.file_manager.buttons.current_condns();
        if status
            & (FactoryFileManager::SAV_SERIES_NEEDS_SAVING
                + FactoryFileManager::SAV_EDITOR_NEEDS_SAVING)
            != 0
        {
            if status & FactoryFileManager::SAV_IS_SAVEABLE != 0 {
                let buttons = [
                    ("Cancel", gtk::ResponseType::Other(0)),
                    ("Save and Continue", gtk::ResponseType::Other(1)),
                    ("Continue Discarding Changes", gtk::ResponseType::Other(2)),
                ];
                match self.ask_question("There are unsaved changes!", None, &buttons) {
                    gtk::ResponseType::Other(0) => return false,
                    gtk::ResponseType::Other(1) => {
                        let o_path = self.file_manager.current_file_path.borrow().clone();
                        if let Some(path) = o_path {
                            if let Err(err) = self.write_to_file(&path) {
                                self.report_error("Failed to save file", &err);
                                return false;
                            }
                        } else if let Some(path) =
                            self.ask_file_path(Some("Save as: "), None, false)
                        {
                            if let Err(err) = self.write_to_file(path) {
                                self.report_error("Failed to save file", &err);
                                return false;
                            }
                        } else {
                            return false;
                        };
                        return true;
                    }
                    _ => return true,
                }
            } else {
                let buttons = &[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Continue Discarding Changes", gtk::ResponseType::Accept),
                ];
                return self.ask_question("There are unsaved changes!", None, buttons)
                    == gtk::ResponseType::Accept;
            }
        };
        true
    }

    fn reset(&self) {
        if self.ok_to_reset() {
            self.unguarded_reset();
        }
    }

    fn unguarded_reset(&self) {
        self.paint_editor.hard_reset();
        self.proprietor_entry.set_text("");
        self.series_name_entry.set_text("");
        self.paint_series.borrow_mut().remove_all();
        self.hue_wheel.remove_all();
        self.list_view.remove_all();
        self.file_manager
            .set_current_file_path(Option::<&str>::None);
        let new_digest = self.paint_series.borrow().digest().expect("unrecoverable");
        *self.saved_series_digest.borrow_mut() = new_digest;
        self.update_series_needs_saving();
        self.update_editor_needs_saving();
        self.update_saveability();
    }

    fn load(&self) {
        if let Some(path) = self.ask_file_path(Some("Load from: "), None, true) {
            match File::open(&path) {
                Ok(mut file) => match BasicPaintSeriesSpec::<f64>::read(&mut file) {
                    Ok(new_series) => {
                        if self.ok_to_reset() {
                            self.unguarded_reset();
                            let id = new_series.series_id();
                            self.proprietor_entry.set_text(id.proprietor());
                            self.series_name_entry.set_text(id.series_name());
                            {
                                let mut series = self.paint_series.borrow_mut();
                                for paint in new_series.paints() {
                                    series.add(paint);
                                    self.hue_wheel.add_item(paint.coloured_shape());
                                    let row = paint.row(&self.attributes, &self.characteristics);
                                    self.list_view.add_row(&row);
                                }
                            }
                            self.file_manager.set_current_file_path(Some(path));
                            let new_digest =
                                self.paint_series.borrow().digest().expect("unrecoverable");
                            *self.saved_series_digest.borrow_mut() = new_digest;
                            self.update_series_needs_saving();
                            self.update_editor_needs_saving();
                            self.update_saveability();
                        }
                    }
                    Err(err) => self.report_error("Bad data.", &err),
                },
                Err(err) => self.report_error("Failed to open file.", &err),
            }
        }
    }
}
