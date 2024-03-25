// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use pw_gtk_ext::{
    gtk::{self, prelude::*},
    gtkx::{
        list::{ListViewWithPopUpMenu, ListViewWithPopUpMenuBuilder},
        menu::MenuItemSpec,
        paned::RememberPosition,
    },
    sav_state::SAV_HOVER_OK,
    wrapper::*,
};

use colour_math::{beigui::hue_wheel::MakeColouredShape, ColourBasics, ScalarAttribute};
use colour_math_gtk::hue_wheel::{GtkHueWheel, GtkHueWheelBuilder};

use apaint::{
    characteristics::CharacteristicType, legacy::legacy_series::SeriesPaintSeriesSpec00,
    series::BasicPaintSpec, series::SeriesPaintSeriesSpec, BasicPaintIfce,
};

use crate::{
    list::{BasicPaintListViewSpec, PaintListRow},
    spec_edit::BasicPaintSpecEditor,
    storage::{StorageManager, StorageManagerBuilder},
};
use apaint::legacy::read_legacy_paint_series_spec;

#[derive(PWO, Wrapper)]
pub struct BasicPaintFactory {
    vbox: gtk::Box,
    file_manager: Rc<StorageManager>,
    paint_editor: Rc<BasicPaintSpecEditor>,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ListViewWithPopUpMenu>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    paint_series: RefCell<SeriesPaintSeriesSpec>,
    proprietor_entry: gtk::Entry,
    series_name_entry: gtk::Entry,
}

impl BasicPaintFactory {
    fn update_saveability(&self) {
        let series = self.paint_series.borrow();
        let series_id = series.series_id();
        self.file_manager.update_session_is_saveable(
            !series_id.proprietor().is_empty() && !series_id.series_name().is_empty(),
        );
    }

    fn update_series_needs_saving(&self) {
        let digest = self.paint_series.borrow().digest().expect("unrecoverable");
        self.file_manager.update_session_needs_saving(&digest);
    }

    fn update_editor_needs_saving(&self) {
        self.file_manager
            .update_tool_needs_saving(self.paint_editor.has_unsaved_changes());
    }

    fn do_add_paint_work(&self, paint_spec: &BasicPaintSpec) {
        if let Some(old_paint) = self.paint_series.borrow_mut().add(paint_spec) {
            self.hue_wheel.remove_item(old_paint.id());
            self.list_view.remove_row(old_paint.id());
        }
        self.hue_wheel.add_item(paint_spec.coloured_shape());
        let row = paint_spec.row(&self.attributes, &self.characteristics);
        self.list_view.add_row(&row);
    }

    fn do_remove_paint_work(&self, id: &str) -> apaint::Result<()> {
        self.paint_series.borrow_mut().remove(id)?;
        self.hue_wheel.remove_item(id);
        self.list_view.remove_row(id);
        Ok(())
    }

    fn add_paint(&self, paint_spec: &BasicPaintSpec) {
        self.do_add_paint_work(paint_spec);
        self.update_series_needs_saving();
        self.update_editor_needs_saving();
    }

    fn remove_paint(&self, id: &str) {
        let question = format!("Confirm remove '{id}'?");
        if self.ask_confirm_action(&question, None) {
            self.do_remove_paint_work(id).expect("should be successful");
            self.paint_editor.un_edit(id);
            self.update_series_needs_saving();
        }
    }

    fn replace_paint(&self, id: &str, paint_spec: &BasicPaintSpec) {
        // should not be called if paint has been removed after being chosen for edit
        self.do_remove_paint_work(id)
            .expect("should not be called if paint has been removed");
        self.do_add_paint_work(paint_spec);
        self.update_series_needs_saving();
        self.update_editor_needs_saving();
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
        let mut spec = BasicPaintSpec::new(&paint.hcv(), paint.id());
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
        self.update_editor_needs_saving();
    }

    fn write_to_file<Q: AsRef<Path>>(&self, path: Q) -> apaint::Result<Vec<u8>> {
        let path: &Path = path.as_ref();
        let mut file = File::create(path)?;
        let new_digest = self.paint_series.borrow_mut().write(&mut file)?;
        Ok(new_digest)
    }

    fn reset(&self) -> apaint::Result<Vec<u8>> {
        self.unguarded_reset();
        let digest = self.paint_series.borrow().digest().expect("unrecoverable");
        Ok(digest)
    }

    fn unguarded_reset(&self) {
        self.paint_editor.hard_reset();
        self.proprietor_entry.set_text("");
        self.series_name_entry.set_text("");
        self.paint_series.borrow_mut().remove_all();
        self.hue_wheel.remove_all();
        self.list_view.remove_all();
        self.update_series_needs_saving();
    }

    fn load<Q: AsRef<Path>>(&self, path: Q) -> apaint::Result<Vec<u8>> {
        let path: &Path = path.as_ref();
        let mut file = File::open(path)?;
        let new_series = match SeriesPaintSeriesSpec::read(&mut file) {
            Ok(series) => series,
            Err(_) => {
                let mut file = File::open(path)?;
                match SeriesPaintSeriesSpec00::<f64>::read(&mut file) {
                    Ok(series) => series,
                    Err(err) => match &err {
                        apaint::Error::SerdeJsonError(_) => {
                            let mut file = File::open(path)?;
                            if let Ok(series) = read_legacy_paint_series_spec(&mut file) {
                                series
                            } else {
                                return Err(err);
                            }
                        }
                        _ => return Err(err),
                    },
                }
            }
        };
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
        self.update_series_needs_saving();
        self.update_editor_needs_saving();
        let digest = self.paint_series.borrow().digest().expect("unrecoverable");
        Ok(digest)
    }

    pub fn needs_saving(&self) -> bool {
        self.file_manager.needs_saving()
    }
}

#[derive(Default)]
pub struct BasicPaintFactoryBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
}

impl BasicPaintFactoryBuilder {
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

    pub fn build(&self) -> Rc<BasicPaintFactory> {
        let menu_items: &[(&'static str, MenuItemSpec, u64)] = &[
            (
                "edit",
                ("Edit", None, Some("Edit the indicated paint")).into(),
                SAV_HOVER_OK,
            ),
            (
                "remove",
                (
                    "Remove",
                    None,
                    Some("Remove the indicated paint from the series."),
                )
                    .into(),
                SAV_HOVER_OK,
            ),
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
        let paint_editor = BasicPaintSpecEditor::new(&self.attributes, &self.characteristics);
        let hue_wheel = GtkHueWheelBuilder::new()
            .menu_item_specs(menu_items)
            .attributes(&self.attributes)
            .build();
        let paint_list_spec = BasicPaintListViewSpec::new(&self.attributes, &self.characteristics);
        let list_view = ListViewWithPopUpMenuBuilder::new()
            .menu_items(menu_items.to_vec())
            .build(&paint_list_spec);
        let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
        scrolled_window.add(list_view.pwo());
        let notebook = gtk::NotebookBuilder::new().build();
        notebook.add(&scrolled_window);
        notebook.set_tab_label_text(&scrolled_window, "Paint List");
        notebook.add(hue_wheel.pwo());
        notebook.set_tab_label_text(hue_wheel.pwo(), "Hue/Attribute Wheel");
        vbox.pack_start(&notebook, true, true, 0);
        paned.add1(&vbox);
        paned.add2(paint_editor.pwo());
        paned.set_position_from_recollections("basic paint factory h paned position", 200);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let file_manager = StorageManagerBuilder::new()
            .last_file_key("factory::series_paints")
            .tooltip_text(
                "reset",
                "Clear the editor in preparation for creating a new collection",
            )
            .tooltip_text("load", "Load a paint collection from a file for editing.")
            .tooltip_text("save", "Save the current editor content to the current file (or to a nominated file if there's no current file).")
            .tooltip_text("save as", "Save the current editor content to a nominated file which will become the current file.")
            .build();
        vbox.pack_start(file_manager.pwo(), false, false, 0);
        vbox.pack_start(&paned, true, true, 0);
        let bpf = Rc::new(BasicPaintFactory {
            vbox,
            file_manager,
            paint_editor,
            hue_wheel,
            list_view,
            attributes: self.attributes.to_vec(),
            characteristics: self.characteristics.to_vec(),
            paint_series: RefCell::new(SeriesPaintSeriesSpec::default()),
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
            .connect_popup_menu_item("edit", move |id, _| bpf_c.edit_paint(&id.unwrap()));

        let bpf_c = Rc::clone(&bpf);
        bpf.hue_wheel
            .connect_popup_menu_item("remove", move |id| bpf_c.remove_paint(id));

        let bpf_c = Rc::clone(&bpf);
        bpf.list_view
            .connect_popup_menu_item("remove", move |id, _| bpf_c.remove_paint(&id.unwrap()));

        let bpf_c = Rc::clone(&bpf);
        bpf.proprietor_entry.connect_changed(move |entry| {
            let text = entry.get_text();
            bpf_c.paint_series.borrow_mut().set_proprietor(&text);
            bpf_c.update_saveability();
            bpf_c.update_series_needs_saving();
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.series_name_entry.connect_changed(move |entry| {
            let text = entry.get_text();
            bpf_c.paint_series.borrow_mut().set_series_name(&text);
            bpf_c.update_saveability();
            bpf_c.update_series_needs_saving();
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager.connect_reset(move || bpf_c.reset());

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager.connect_load(move |p| bpf_c.load(p));

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager
            .connect_save(move |p| bpf_c.write_to_file(p));

        bpf
    }
}
