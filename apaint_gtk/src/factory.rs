// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::Serialize;

use gtk::prelude::*;
use pw_gix::wrapper::*;

use pw_gix::gtkx::paned::RememberPosition;
use pw_gix::sav_state::{ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled};

use colour_math::ScalarAttribute;

use apaint::{
    characteristics::CharacteristicType, hue_wheel::MakeColouredShape, series::PaintSeries,
    BasicPaintIfce, BasicPaintSpec, FromSpec,
};

use apaint_gtk_boilerplate::{Wrapper, PWO};

use crate::hue_wheel::GtkHueWheel;
use crate::list::{ColouredItemListView, PaintListHelper};
use crate::spec_edit::BasicPaintSpecEditor;
use crate::{icon_image, SAV_HAS_CHOSEN_ITEM};
use std::fs::File;

#[derive(PWO)]
struct FactoryFileManager {
    hbox: gtk::Box,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    current_file_path: RefCell<Option<PathBuf>>,
}

impl FactoryFileManager {
    const SAV_HAS_CURRENT_FILE: u64 = 1 << 0;
    const SAV_IS_SAVEABLE: u64 = 1 << 1;
    const SAV_EDITOR_NEEDS_SAVING: u64 = 1 << 2;
    const SAV_SERIES_NEEDS_SAVING: u64 = 1 << 3;

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
        new_colln_btn.set_image(Some(&icon_image::colln_new_image(24)));
        buttons.add_widget("new_colln", &new_colln_btn, 0);
        hbox.pack_start(&new_colln_btn, false, false, 0);

        let save_colln_btn = gtk::ButtonBuilder::new()
            .tooltip_text("Save the current editor content to the current file.")
            .build();
        // TODO: change setting of image when ButtonBuilder interface is fixed.
        save_colln_btn.set_image(Some(&icon_image::colln_save_image(24)));
        buttons.add_widget(
            "save_colln",
            &save_colln_btn,
            Self::SAV_HAS_CURRENT_FILE + Self::SAV_IS_SAVEABLE,
        );
        hbox.pack_start(&save_colln_btn, false, false, 0);

        let save_as_colln_btn = gtk::ButtonBuilder::new()
            .tooltip_text("Save the current editor content to a nominated file.")
            .build();
        // TODO: change setting of image when ButtonBuilder interface is fixed.
        save_as_colln_btn.set_image(Some(&icon_image::colln_save_as_image(24)));
        buttons.add_widget("save_as_colln", &save_as_colln_btn, Self::SAV_IS_SAVEABLE);
        hbox.pack_start(&save_as_colln_btn, false, false, 0);

        hbox.show_all();

        Self {
            hbox,
            buttons,
            current_file_path: RefCell::new(None),
        }
    }

    fn set_current_file_path<Q: AsRef<Path>>(&self, path: Option<Q>) {
        let mut condns: u64 = 0;
        let mask: u64 = Self::SAV_HAS_CURRENT_FILE;
        if let Some(path) = path {
            let path: PathBuf = path.as_ref().to_path_buf();
            *self.current_file_path.borrow_mut() = Some(path);
            condns = Self::SAV_HAS_CURRENT_FILE;
        } else {
            *self.current_file_path.borrow_mut() = None;
        }
        self.buttons.update_condns(MaskedCondns { condns, mask });
    }
}

#[derive(PWO, Wrapper)]
pub struct BasicPaintFactory<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + Serialize + 'static,
{
    vbox: gtk::Box,
    file_manager: FactoryFileManager,
    paint_editor: Rc<BasicPaintSpecEditor>,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    paint_list_helper: PaintListHelper,
    paint_series: RefCell<PaintSeries<f64, P>>,
    saved_series_digest: RefCell<Vec<u8>>,
    proprietor_entry: gtk::Entry,
    series_name_entry: gtk::Entry,
}

impl<P> BasicPaintFactory<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + Serialize + 'static,
{
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let menu_items: &[(&str, &str, Option<&gtk::Image>, &str, u64)] = &[(
            "remove",
            "Remove",
            None,
            "Remove the indicated paint from the series.",
            SAV_HAS_CHOSEN_ITEM,
        )];
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
        let paint_editor = BasicPaintSpecEditor::new(attributes, &[]);
        let hue_wheel = GtkHueWheel::new(menu_items, attributes);
        let paint_list_helper = PaintListHelper::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(
            &paint_list_helper.column_types(),
            &paint_list_helper.columns(),
            menu_items,
        );
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
            paint_list_helper,
            paint_series: RefCell::new(PaintSeries::default()),
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
            .get_widget("save_colln")
            .unwrap()
            .connect_clicked(move |_| bpf_c.save());

        let bpf_c = Rc::clone(&bpf);
        bpf.file_manager
            .buttons
            .get_widget("save_as_colln")
            .unwrap()
            .connect_clicked(move |_| bpf_c.save_as());

        bpf
    }

    fn update_saveability(&self) {
        let mut condns: u64 = 0;
        let mask: u64 = FactoryFileManager::SAV_IS_SAVEABLE;
        let series = self.paint_series.borrow();
        let series_id = series.series_id();
        // TODO: take spec editor into account deciding saveability
        if series_id.proprietor().len() > 0 && series_id.series_name().len() > 0 {
            condns = FactoryFileManager::SAV_IS_SAVEABLE;
        }
        self.file_manager
            .buttons
            .update_condns(MaskedCondns { condns, mask });
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
    }

    fn update_editor_needs_saving(&self) {
        let mut condns: u64 = 0;
        let mask = FactoryFileManager::SAV_EDITOR_NEEDS_SAVING;
        if self.paint_editor.has_unsaved_changes() {
            condns += FactoryFileManager::SAV_EDITOR_NEEDS_SAVING;
        }
        self.file_manager
            .buttons
            .update_condns(MaskedCondns { condns, mask });
    }

    fn do_add_paint_work(&self, paint_spec: &BasicPaintSpec<f64>) {
        let paint = P::from_spec(paint_spec);
        if let Some(old_paint) = self.paint_series.borrow_mut().add(&paint) {
            self.hue_wheel.remove_item(old_paint.id());
            self.list_view.remove_row(old_paint.id());
        }
        self.hue_wheel.add_item(paint.coloured_shape());
        let row = self.paint_list_helper.row(&paint);
        self.list_view.add_row(&row);
    }

    fn do_remove_paint_work(&self, id: &str) {
        self.paint_series.borrow_mut().remove(id);
        self.hue_wheel.remove_item(id);
        self.list_view.remove_row(id);
    }

    fn add_paint(&self, paint_spec: &BasicPaintSpec<f64>) {
        self.do_add_paint_work(paint_spec);
        self.update_series_needs_saving();
    }

    fn remove_paint(&self, id: &str) {
        // TODO: put in a "confirm remove" dialog here
        self.do_remove_paint_work(id);
        self.update_series_needs_saving();
    }

    fn replace_paint(&self, id: &str, paint_spec: &BasicPaintSpec<f64>) {
        self.do_remove_paint_work(id);
        self.do_add_paint_work(paint_spec);
        self.update_series_needs_saving();
    }

    fn write_to_file<Q: AsRef<Path>>(&self, path: Q) -> Result<(), apaint::Error> {
        let path: &Path = path.as_ref();
        let mut file = File::create(path)?;
        self.paint_series.borrow_mut().write(&mut file)?;
        self.file_manager.set_current_file_path(Some(path));
        let new_digest = self.paint_series.borrow().digest().expect("unrecoverable");
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
            println!("reset");
            self.paint_editor.hard_reset();
            // TODO: reset series_id
            // TODO: reset the paint series to empty
            // TODO: reset the saved file name
            self.update_series_needs_saving();
        }
    }
}
