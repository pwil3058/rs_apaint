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

#[derive(PWO, Wrapper)]
struct FactoryFileManager {
    hbox: gtk::Box,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
    current_file_path: RefCell<Option<PathBuf>>,
}

impl FactoryFileManager {
    const SAV_HAS_CURRENT_FILE: u64 = 1 << 0;

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
        buttons.add_widget("save_colln", &save_colln_btn, Self::SAV_HAS_CURRENT_FILE);
        hbox.pack_start(&save_colln_btn, false, false, 0);

        let save_as_colln_btn = gtk::ButtonBuilder::new()
            .tooltip_text("Save the current editor content to a nominated file.")
            .build();
        // TODO: change setting of image when ButtonBuilder interface is fixed.
        save_as_colln_btn.set_image(Some(&icon_image::colln_save_as_image(24)));
        buttons.add_widget("save_as_colln", &save_as_colln_btn, 0);
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

#[derive(PWO)]
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
            proprietor_entry,
            series_name_entry,
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.paint_editor
            .connect_add_action(move |spec| bpf_c.add_paint(spec));

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
            }
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.series_name_entry.connect_changed(move |entry| {
            if let Some(text) = entry.get_text() {
                bpf_c.paint_series.borrow_mut().set_series_name(&text);
            }
        });

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

    fn add_paint(&self, paint_spec: &BasicPaintSpec<f64>) {
        let paint = P::from_spec(paint_spec);
        if let Some(old_paint) = self.paint_series.borrow_mut().add(&paint) {
            self.hue_wheel.remove_item(old_paint.id());
            self.list_view.remove_row(old_paint.id());
        }
        self.hue_wheel.add_item(paint.coloured_shape());
        let row = self.paint_list_helper.row(&paint);
        self.list_view.add_row(&row);
    }

    fn remove_paint(&self, id: &str) {
        // TODO: put in a "confirm remove" dialog here
        self.paint_series.borrow_mut().remove(id);
        self.hue_wheel.remove_item(id);
        self.list_view.remove_row(id);
    }

    fn write_to_file<Q: AsRef<Path>>(&self, path: Q) -> Result<(), apaint::Error> {
        let path: &Path = path.as_ref();
        let mut file = File::create(path)?;
        self.paint_series.borrow_mut().write(&mut file)?;
        self.file_manager.set_current_file_path(Some(path));
        Ok(())
    }

    fn save(&self) {
        if let Some(path) = self.file_manager.current_file_path.borrow().as_ref() {
            if let Err(err) = self.write_to_file(path) {
                self.file_manager.report_error("Problem saving file", &err);
            }
        } else {
            panic!("programming error: save() should not have been called.")
        }
    }

    fn save_as(&self) {
        // TODO: use last dir data option
        if let Some(path) = self
            .file_manager
            .ask_file_path(Some("Save as: "), None, false)
        {
            if let Err(err) = self.write_to_file(path) {
                self.file_manager.report_error("Problem saving file", &err);
            }
        }
    }
}
