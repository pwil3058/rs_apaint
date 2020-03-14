// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk::prelude::*;

use pw_gix::{
    gtkx::{
        dialog::dialog_user::TopGtkWindow,
        menu::MenuItemSpec,
        notebook::{TabRemoveLabel, TabRemoveLabelInterface},
        paned::RememberPosition,
    },
    recollections::{recall, remember},
    sav_state::{ChangedCondnsNotifier, MaskedCondns, SAV_HOVER_OK},
    wrapper::*,
};

use colour_math::attributes::hue_wheel::MakeColouredShape;
use colour_math_gtk::hue_wheel::GtkHueWheel;

use apaint::{
    characteristics::CharacteristicType,
    series::{SeriesId, SeriesPaint, SeriesPaintFinder, SeriesPaintSeries, SeriesPaintSeriesSpec},
};

use crate::{
    colour::{ScalarAttribute, RGB},
    icon_image::{paint_standard_load_image, series_paint_load_image},
    list::{BasicPaintListViewSpec, ColouredItemListView, PaintListRow},
};

pub mod display;

use crate::series::display::*;

type PaintActionCallback = Box<dyn Fn(Rc<SeriesPaint<f64>>)>;

#[derive(PWO, Wrapper)]
struct SeriesPage {
    paned: gtk::Paned,
    paint_series: SeriesPaintSeries<f64>,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    callbacks: RefCell<HashMap<String, Vec<PaintActionCallback>>>,
}

impl SeriesPage {
    fn new(
        paint_series: SeriesPaintSeries<f64>,
        menu_items: &[MenuItemSpec],
        attributes: &[ScalarAttribute],
        characteristics: &[CharacteristicType],
    ) -> Rc<Self> {
        let paned = gtk::PanedBuilder::new().build();
        paned.set_position_from_recollections("SeriesPage:paned_position", 200);
        let hue_wheel = GtkHueWheel::new(menu_items, attributes);
        let list_spec = BasicPaintListViewSpec::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(&list_spec, menu_items);
        for paint in paint_series.paints() {
            hue_wheel.add_item(paint.coloured_shape());
            let row = paint.row(attributes, characteristics);
            list_view.add_row(&row);
        }
        let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
        scrolled_window.add(&list_view.pwo());
        paned.add1(&hue_wheel.pwo());
        paned.add2(&scrolled_window);
        let sp = Rc::new(Self {
            paned,
            paint_series,
            hue_wheel,
            list_view,
            callbacks: RefCell::new(HashMap::new()),
        });
        for menu_item in menu_items.iter() {
            let sp_c = Rc::clone(&sp);
            let item_name_c = menu_item.name().to_string();
            sp.hue_wheel
                .connect_popup_menu_item(menu_item.name(), move |id| {
                    sp_c.invoke_named_callback(&item_name_c, id)
                });
            let sp_c = Rc::clone(&sp);
            let item_name_c = menu_item.name().to_string();
            sp.list_view
                .connect_popup_menu_item(menu_item.name(), move |id| {
                    sp_c.invoke_named_callback(&item_name_c, id)
                });
            sp.callbacks
                .borrow_mut()
                .insert(menu_item.name().to_string(), vec![]);
        }

        sp
    }

    fn series_id(&self) -> &Rc<SeriesId> {
        self.paint_series.series_id()
    }

    fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.hue_wheel.update_popup_condns(changed_condns);
        self.list_view.update_popup_condns(changed_condns);
    }

    fn connect_popup_menu_item<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(
        &self,
        name: &str,
        callback: F,
    ) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    fn invoke_named_callback(&self, item: &str, id: &str) {
        if let Some(paint) = self.paint_series.find(id) {
            for callback in self
                .callbacks
                .borrow()
                .get(item)
                .expect("invalid name")
                .iter()
            {
                callback(Rc::clone(paint))
            }
        }
    }

    fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.hue_wheel.set_target_rgb(rgb);
    }
}

#[derive(PWO, Wrapper)]
struct SeriesBinder {
    notebook: gtk::Notebook,
    pages: RefCell<Vec<(Rc<SeriesPage>, PathBuf)>>,
    menu_items: Vec<MenuItemSpec>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: RefCell<Option<RGB>>,
    callbacks: RefCell<HashMap<String, Vec<PaintActionCallback>>>,
    loaded_files_data_path: Option<PathBuf>,
}

impl SeriesBinder {
    fn new(
        menu_items: &[MenuItemSpec],
        attributes: &[ScalarAttribute],
        characteristics: &[CharacteristicType],
        loaded_files_data_path: Option<PathBuf>,
    ) -> Rc<Self> {
        let notebook = gtk::NotebookBuilder::new().enable_popup(true).build();
        let pages = RefCell::new(vec![]);
        let mut hash_map: HashMap<String, Vec<PaintActionCallback>> = HashMap::new();
        for menu_item in menu_items.iter() {
            let item_name = menu_item.name();
            hash_map.insert(item_name.to_string(), vec![]);
        }
        let callbacks = RefCell::new(hash_map);
        let binder = Rc::new(Self {
            notebook,
            pages,
            menu_items: menu_items.to_vec(),
            attributes: attributes.to_vec(),
            characteristics: characteristics.to_vec(),
            target_rgb: RefCell::new(None),
            callbacks,
            loaded_files_data_path,
        });

        for path_buf in &binder.read_loaded_file_paths() {
            if let Err(err) = binder.add_series_from_file(&path_buf) {
                binder.report_error("Error preloading:", &err);
            }
        }
        binder.write_loaded_file_paths();

        binder
    }

    fn binary_search_series_id(&self, sid: &Rc<SeriesId>) -> Result<usize, usize> {
        self.pages
            .borrow()
            .binary_search_by_key(&sid, |(page, _)| page.series_id())
    }

    fn find_file_path(&self, path: &Path) -> Option<usize> {
        for (index, (_, page_path)) in self.pages.borrow().iter().enumerate() {
            if path == page_path {
                return Some(index);
            }
        }
        None
    }

    fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        for (page, _) in self.pages.borrow().iter() {
            page.update_popup_condns(changed_condns)
        }
    }

    fn connect_popup_menu_item<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(
        &self,
        name: &str,
        callback: F,
    ) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    fn invoke_named_callback(&self, item: &str, paint: Rc<SeriesPaint<f64>>) {
        for callback in self
            .callbacks
            .borrow()
            .get(item)
            .expect("invalid name")
            .iter()
        {
            callback(Rc::clone(&paint))
        }
    }

    fn set_target_rgb(&self, rgb: Option<&RGB>) {
        if let Some(rgb) = rgb {
            *self.target_rgb.borrow_mut() = Some(*rgb);
            for (page, _) in self.pages.borrow().iter() {
                page.set_target_rgb(Some(rgb));
            }
        } else {
            *self.target_rgb.borrow_mut() = None;
            for (page, _) in self.pages.borrow().iter() {
                page.set_target_rgb(None);
            }
        }
    }

    fn remove_series_at_index(&self, index: usize) {
        let page = self.pages.borrow_mut().remove(index);
        let page_num = self.notebook.page_num(&page.0.pwo());
        self.notebook.remove_page(page_num);
    }

    fn remove_series(&self, series_id: &Rc<SeriesId>) {
        let question = format!("Confirm remove '{}'?", series_id);
        if self.ask_confirm_action(&question, None) {
            if let Ok(index) = self.binary_search_series_id(&series_id) {
                self.remove_series_at_index(index)
            } else {
                panic!("attempt to remove non existent series")
            }
        }
    }

    fn read_loaded_file_paths(&self) -> Vec<PathBuf> {
        if let Some(loaded_files_data_path) = &self.loaded_files_data_path {
            if loaded_files_data_path.exists() {
                let mut file = File::open(&loaded_files_data_path).expect("unrecoverable");
                let mut string = String::new();
                file.read_to_string(&mut string).expect("unrecoverable");
                return string.lines().map(PathBuf::from).collect();
            }
        }
        vec![]
    }

    fn write_loaded_file_paths(&self) {
        if let Some(loaded_files_data_path) = &self.loaded_files_data_path {
            let mut string = String::new();
            for (_, path_buf) in self.pages.borrow().iter() {
                string += (pw_pathux::path_to_string(&path_buf) + "\n").as_str();
            }
            let mut file = File::create(loaded_files_data_path).expect("unrecoverable");
            file.write_all(&string.into_bytes()).expect("unrecoverable");
        }
    }
}

trait RcSeriesBinder {
    fn add_series(
        &self,
        new_series: SeriesPaintSeries<f64>,
        path: &Path,
    ) -> Result<(), crate::Error>;
    fn add_series_from_file(&self, path: &Path) -> Result<(), crate::Error>;
}

impl RcSeriesBinder for Rc<SeriesBinder> {
    fn add_series(
        &self,
        new_series: SeriesPaintSeries<f64>,
        path: &Path,
    ) -> Result<(), crate::Error> {
        match self.binary_search_series_id(&new_series.series_id()) {
            Ok(_) => Err(crate::Error::GeneralError(format!(
                "{}: Series already in binder",
                &new_series.series_id()
            ))),
            Err(index) => {
                let l_text = format!(
                    "{}\n{}",
                    new_series.series_id().series_name(),
                    new_series.series_id().proprietor(),
                );
                let tt_text = format!(
                    "Remove {} ({}) from the tool kit",
                    new_series.series_id().series_name(),
                    new_series.series_id().proprietor(),
                );
                let label = TabRemoveLabel::create(Some(l_text.as_str()), Some(&tt_text.as_str()));
                let self_c = Rc::clone(self);
                let sid = new_series.series_id().clone();
                label.connect_remove_page(move || self_c.remove_series(&sid));
                let l_text = format!(
                    "{} ({})",
                    new_series.series_id().series_name(),
                    new_series.series_id().proprietor(),
                );
                let menu_label = gtk::Label::new(Some(l_text.as_str()));
                let new_page = SeriesPage::new(
                    new_series,
                    &self.menu_items,
                    &self.attributes,
                    &self.characteristics,
                );
                if let Some(rgb) = &self.target_rgb.borrow().as_ref() {
                    new_page.set_target_rgb(Some(rgb));
                };
                for menu_item in self.menu_items.iter() {
                    let self_c = Rc::clone(self);
                    let item_name_c = menu_item.name().to_string();
                    new_page.connect_popup_menu_item(menu_item.name(), move |paint| {
                        self_c.invoke_named_callback(&item_name_c, paint)
                    });
                }
                self.notebook.insert_page_menu(
                    &new_page.pwo(),
                    Some(&label.pwo()),
                    Some(&menu_label),
                    Some(index as u32),
                );
                self.notebook.show_all();
                self.pages
                    .borrow_mut()
                    .insert(index, (new_page, path.to_path_buf()));
                Ok(())
            }
        }
    }

    fn add_series_from_file(&self, path: &Path) -> Result<(), crate::Error> {
        if self.find_file_path(path).is_some() {
            let msg = format!("{}: is already loaded", path.to_string_lossy());
            return Err(crate::Error::DuplicateFile(msg));
        }
        let mut file = File::open(path)?;
        let new_series_spec = SeriesPaintSeriesSpec::<f64>::read(&mut file)?;
        self.add_series((&new_series_spec).into(), path)?;
        Ok(())
    }
}

impl SeriesPaintFinder<f64> for SeriesBinder {
    fn get_series_paint(
        &self,
        paint_id: &str,
        series_id: Option<&SeriesId>,
    ) -> apaint::Result<Rc<SeriesPaint<f64>>> {
        if let Some(series_id) = series_id {
            let bsr = self
                .pages
                .borrow()
                .binary_search_by_key(&series_id, |(page, _)| page.series_id());
            match bsr {
                Ok(index) => match self.pages.borrow()[index].0.paint_series.find(paint_id) {
                    Some(paint) => Ok(Rc::clone(paint)),
                    None => Err(apaint::Error::UnknownSeriesPaint(
                        series_id.clone(),
                        paint_id.to_string(),
                    )),
                },
                Err(_) => Err(apaint::Error::UnknownSeries(series_id.clone())),
            }
        } else {
            for page in self.pages.borrow().iter() {
                if let Some(paint) = page.0.paint_series.find(paint_id) {
                    return Ok(Rc::clone(paint));
                }
            }
            Err(apaint::Error::NotFound(paint_id.to_string()))
        }
    }
}

#[derive(PWO, Wrapper)]
pub struct PaintSeriesManager {
    vbox: gtk::Box,
    binder: Rc<SeriesBinder>,
    display_dialog_manager: Rc<PaintDisplayDialogManager<gtk::Box>>,
    add_paint_callbacks: RefCell<Vec<PaintActionCallback>>,
}

impl PaintSeriesManager {
    fn load_series_from_file(&self) -> Result<(), crate::Error> {
        let last_file = recall("PaintSeriesManager::last_loaded_file");
        let last_file = if let Some(ref text) = last_file {
            Some(text.as_str())
        } else {
            None
        };
        if let Some(path) = self.ask_file_path(Some("Collection File Name:"), last_file, true) {
            let abs_path = pw_pathux::expand_home_dir_or_mine(&path).canonicalize()?;
            self.binder.add_series_from_file(&abs_path)?;
            let path_text = pw_pathux::path_to_string(&abs_path);
            remember("PaintSeriesManager::last_loaded_file", &path_text);
            self.binder.write_loaded_file_paths();
        };
        Ok(())
    }

    fn display_paint_information(&self, paint: &Rc<SeriesPaint<f64>>) {
        self.display_dialog_manager.display_paint(paint);
    }

    fn inform_add_paint(&self, paint: &Rc<SeriesPaint<f64>>) {
        for callback in self.add_paint_callbacks.borrow().iter() {
            callback(Rc::clone(paint));
        }
    }

    pub fn connect_add_paint<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(&self, callback: F) {
        self.add_paint_callbacks
            .borrow_mut()
            .push(Box::new(callback));
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.binder.set_target_rgb(rgb);
        self.display_dialog_manager.set_target_rgb(rgb);
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.binder.update_popup_condns(changed_condns);
    }
}

impl SeriesPaintFinder<f64> for PaintSeriesManager {
    fn get_series_paint(
        &self,
        paint_id: &str,
        series_id: Option<&SeriesId>,
    ) -> apaint::Result<Rc<SeriesPaint<f64>>> {
        self.binder.get_series_paint(paint_id, series_id)
    }
}

#[derive(Default)]
pub struct PaintSeriesManagerBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    loaded_files_data_path: Option<PathBuf>,
    change_notifier: Rc<ChangedCondnsNotifier>,
}

impl PaintSeriesManagerBuilder {
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

    pub fn change_notifier(&mut self, change_notifier: &Rc<ChangedCondnsNotifier>) -> &mut Self {
        self.change_notifier = Rc::clone(change_notifier);
        self
    }

    pub fn loaded_files_data_path(&mut self, path: &Path) -> &mut Self {
        self.loaded_files_data_path = Some(path.to_path_buf());
        self
    }

    pub fn build(&self) -> Rc<PaintSeriesManager> {
        let menu_items = &[
            (
                "info",
                "Paint Information",
                None,
                "Display information for the indicated paint",
                SAV_HOVER_OK,
            )
                .into(),
            (
                "add",
                "Add",
                None,
                "Add the indicated paint to the mixer/palette",
                SAV_HOVER_OK,
            )
                .into(),
        ];
        let binder = SeriesBinder::new(
            menu_items,
            &self.attributes,
            &self.characteristics,
            self.loaded_files_data_path.clone(),
        );
        let load_file_btn = gtk::ButtonBuilder::new()
            .image(&series_paint_load_image(24).upcast::<gtk::Widget>())
            .tooltip_text("Load a paint series from a file.")
            .build();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&load_file_btn, false, false, 0);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&binder.pwo(), true, true, 0);
        vbox.show_all();
        let display_dialog_manager = PaintDisplayDialogManagerBuilder::new(&vbox)
            .attributes(&self.attributes)
            .characteristics(&self.characteristics)
            .change_notifier(&self.change_notifier)
            .buttons(&[(0, "Add", Some("Add this paint to the mixer/palette"), 0)])
            .build();

        let psm = Rc::new(PaintSeriesManager {
            vbox,
            binder,
            display_dialog_manager,
            add_paint_callbacks: RefCell::new(vec![]),
        });

        let psm_c = Rc::clone(&psm);
        psm.binder
            .connect_popup_menu_item("info", move |paint| psm_c.display_paint_information(&paint));

        let psm_c = Rc::clone(&psm);
        psm.binder
            .connect_popup_menu_item("add", move |paint| psm_c.inform_add_paint(&paint));

        let psm_c = Rc::clone(&psm);
        psm.display_dialog_manager
            .connect_action_button(0, move |paint| psm_c.inform_add_paint(&paint));

        let psm_c = Rc::clone(&psm);
        load_file_btn.connect_clicked(move |_| {
            if let Err(err) = psm_c.load_series_from_file() {
                psm_c.report_error("Load file failed.", &err);
            }
        });

        psm
    }
}

#[derive(PWO, Wrapper)]
pub struct PaintStandardsManager {
    vbox: gtk::Box,
    binder: Rc<SeriesBinder>,
    display_dialog_manager: Rc<PaintDisplayDialogManager<gtk::Box>>,
    set_as_target_callbacks: RefCell<Vec<PaintActionCallback>>,
}

impl PaintStandardsManager {
    fn load_series_from_file(&self) -> Result<(), crate::Error> {
        let last_file = recall("PaintStandardsManager::last_loaded_file");
        let last_file = if let Some(ref text) = last_file {
            Some(text.as_str())
        } else {
            None
        };
        if let Some(path) = self.ask_file_path(Some("Paint Standard's File Name:"), last_file, true)
        {
            let abs_path = pw_pathux::expand_home_dir_or_mine(&path).canonicalize()?;
            self.binder.add_series_from_file(&abs_path)?;
            let path_text = pw_pathux::path_to_string(&abs_path);
            remember("PaintStandardsManager::last_loaded_file", &path_text);
            self.binder.write_loaded_file_paths();
        };
        Ok(())
    }

    fn display_paint_information(&self, paint: &Rc<SeriesPaint<f64>>) {
        self.display_dialog_manager.display_paint(paint);
    }

    fn inform_set_as_target(&self, paint: &Rc<SeriesPaint<f64>>) {
        for callback in self.set_as_target_callbacks.borrow().iter() {
            callback(Rc::clone(paint));
        }
    }

    pub fn connect_set_as_target<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(&self, callback: F) {
        self.set_as_target_callbacks
            .borrow_mut()
            .push(Box::new(callback));
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.binder.update_popup_condns(changed_condns);
    }
}

#[derive(Default)]
pub struct PaintStandardsManagerBuilder {
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    loaded_files_data_path: Option<PathBuf>,
    change_notifier: Rc<ChangedCondnsNotifier>,
}

impl PaintStandardsManagerBuilder {
    // TODO: Turn off set as target from dialog when target already set
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

    pub fn change_notifier(&mut self, change_notifier: &Rc<ChangedCondnsNotifier>) -> &mut Self {
        self.change_notifier = Rc::clone(change_notifier);
        self
    }

    pub fn loaded_files_data_path(&mut self, path: &Path) -> &mut Self {
        self.loaded_files_data_path = Some(path.to_path_buf());
        self
    }

    pub fn build(&self) -> Rc<PaintStandardsManager> {
        let menu_items = &[
            (
                "info",
                "Paint Information",
                None,
                "Display information for the indicated paint",
                SAV_HOVER_OK,
            )
                .into(),
            (
                "set target",
                "Set As Target",
                None,
                "Set the indicated standard as the target in the mixer",
                SAV_HOVER_OK + crate::mixer::targeted::TargetedPaintMixer::SAV_NOT_HAS_TARGET,
            )
                .into(),
        ];
        let binder = SeriesBinder::new(
            menu_items,
            &self.attributes,
            &self.characteristics,
            self.loaded_files_data_path.clone(),
        );
        let load_file_btn = gtk::ButtonBuilder::new()
            .image(&paint_standard_load_image(24).upcast::<gtk::Widget>())
            .tooltip_text("Load a paint standards series from a file.")
            .build();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&load_file_btn, false, false, 0);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&binder.pwo(), true, true, 0);
        vbox.show_all();
        let display_dialog_manager = PaintDisplayDialogManagerBuilder::new(&vbox)
            .attributes(&self.attributes)
            .characteristics(&self.characteristics)
            .change_notifier(&self.change_notifier)
            .buttons(&[(
                0,
                "Set as Target",
                Some("Set this colour as the mixer target"),
                crate::mixer::targeted::TargetedPaintMixer::SAV_NOT_HAS_TARGET,
            )])
            .build();

        let psm = Rc::new(PaintStandardsManager {
            vbox,
            binder,
            display_dialog_manager,
            set_as_target_callbacks: RefCell::new(vec![]),
        });

        let psm_c = Rc::clone(&psm);
        psm.binder
            .connect_popup_menu_item("info", move |paint| psm_c.display_paint_information(&paint));

        let psm_c = Rc::clone(&psm);
        psm.binder
            .connect_popup_menu_item("set target", move |paint| {
                psm_c.inform_set_as_target(&paint)
            });

        let psm_c = Rc::clone(&psm);
        psm.display_dialog_manager
            .connect_action_button(0, move |paint| psm_c.inform_set_as_target(&paint));

        let psm_c = Rc::clone(&psm);
        load_file_btn.connect_clicked(move |_| {
            if let Err(err) = psm_c.load_series_from_file() {
                psm_c.report_error("Load file failed.", &err);
            }
        });

        psm
    }
}
