// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, collections::HashMap, fs::File, path::Path, rc::Rc};

use gtk::prelude::*;

use pw_gix::{
    gtkx::notebook::{TabRemoveLabel, TabRemoveLabelInterface},
    recollections::recall,
    sav_state::{MaskedCondns, SAV_HOVER_OK},
    wrapper::*,
};

use apaint_gtk_boilerplate::{Wrapper, PWO};

use apaint::{
    characteristics::CharacteristicType,
    hue_wheel::MakeColouredShape,
    series::{SeriesPaint, SeriesPaintSeries},
    spec::{BasicPaintSeriesSpec, SeriesId},
};

use crate::icon_image::series_paint_load_image;
use crate::{
    colour::{ScalarAttribute, RGB},
    hue_wheel::GtkHueWheel,
    list::{BasicPaintListViewSpec, ColouredItemListView, PaintListRow},
    managed_menu::MenuItemSpec,
};

#[derive(PWO, Wrapper)]
pub struct SeriesPage {
    paned: gtk::Paned,
    paint_series: SeriesPaintSeries<f64>,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(Rc<SeriesPaint<f64>>)>>>>,
}

impl SeriesPage {
    pub fn new(
        paint_series: SeriesPaintSeries<f64>,
        menu_items: &[MenuItemSpec],
        attributes: &[ScalarAttribute],
        characteristics: &[CharacteristicType],
    ) -> Rc<Self> {
        let paned = gtk::PanedBuilder::new().build();
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

    pub fn series_id(&self) -> &Rc<SeriesId> {
        self.paint_series.series_id()
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.hue_wheel.update_popup_condns(changed_condns);
        self.list_view.update_popup_condns(changed_condns);
    }

    pub fn connect_popup_menu_item<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(
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

    pub fn invoke_named_callback(&self, item: &str, id: &str) {
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

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.hue_wheel.set_target_rgb(rgb);
    }
}

#[derive(PWO, Wrapper)]
pub struct SeriesBinder {
    notebook: gtk::Notebook,
    pages: RefCell<Vec<Rc<SeriesPage>>>,
    menu_items: Vec<MenuItemSpec>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    target_rgb: RefCell<Option<RGB>>,
    callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(Rc<SeriesPaint<f64>>)>>>>,
}

impl SeriesBinder {
    pub fn new(
        menu_items: &[MenuItemSpec],
        attributes: &[ScalarAttribute],
        characteristics: &[CharacteristicType],
    ) -> Rc<Self> {
        let notebook = gtk::NotebookBuilder::new().enable_popup(true).build();
        let pages = RefCell::new(vec![]);
        let mut hash_map: HashMap<String, Vec<Box<dyn Fn(Rc<SeriesPaint<f64>>)>>> = HashMap::new();
        for menu_item in menu_items.iter() {
            let item_name = menu_item.name();
            hash_map.insert(item_name.to_string(), vec![]);
        }
        let callbacks = RefCell::new(hash_map);
        Rc::new(Self {
            notebook,
            pages,
            menu_items: menu_items.to_vec(),
            attributes: attributes.to_vec(),
            characteristics: characteristics.to_vec(),
            target_rgb: RefCell::new(None),
            callbacks,
        })
    }

    fn binary_search_series_id(&self, sid: &Rc<SeriesId>) -> Result<usize, usize> {
        self.pages
            .borrow()
            .binary_search_by_key(&sid, |page| page.series_id())
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        for page in self.pages.borrow().iter() {
            page.update_popup_condns(changed_condns)
        }
    }

    pub fn connect_popup_menu_item<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(
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

    pub fn invoke_named_callback(&self, item: &str, paint: Rc<SeriesPaint<f64>>) {
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

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        if let Some(rgb) = rgb {
            *self.target_rgb.borrow_mut() = Some(*rgb);
            for page in self.pages.borrow().iter() {
                page.set_target_rgb(Some(rgb));
            }
        } else {
            *self.target_rgb.borrow_mut() = None;
            for page in self.pages.borrow().iter() {
                page.set_target_rgb(None);
            }
        }
    }

    fn remove_series_at_index(&self, index: usize) {
        let page = self.pages.borrow_mut().remove(index);
        let page_num = self.notebook.page_num(&page.pwo());
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
}

pub trait RcSeriesBinder {
    fn add_series(&self, new_series: SeriesPaintSeries<f64>) -> Result<(), crate::Error>;
    fn add_series_from_file(&self, path: &Path) -> Result<(), crate::Error>;
}

impl RcSeriesBinder for Rc<SeriesBinder> {
    fn add_series(&self, new_series: SeriesPaintSeries<f64>) -> Result<(), crate::Error> {
        match self.binary_search_series_id(&new_series.series_id()) {
            Ok(_) => Err(crate::Error::GeneralError(
                "Series already in binder".to_string(),
            )),
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
                self.pages.borrow_mut().insert(index, new_page);
                Ok(())
            }
        }
    }

    fn add_series_from_file(&self, path: &Path) -> Result<(), crate::Error> {
        // TODO: check for duplicate files
        let mut file = File::open(path)?;
        let new_series_spec = BasicPaintSeriesSpec::<f64>::read(&mut file)?;
        self.add_series((&new_series_spec).into())?;
        // TODO: adjust date for detecting duplicates
        Ok(())
    }
}

#[derive(PWO, Wrapper)]
pub struct PaintSeriesManager {
    vbox: gtk::Box,
    binder: Rc<SeriesBinder>,
}

impl PaintSeriesManager {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let menu_items = &[(
            "add",
            "Add",
            None,
            "Add the indicated paint to the mixer/palette",
            SAV_HOVER_OK,
        )
            .into()];
        let binder = SeriesBinder::new(menu_items, attributes, characteristics);
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

        let psm = Rc::new(Self { vbox, binder });

        let psm_c = Rc::clone(&psm);
        load_file_btn.connect_clicked(move |_| {
            if let Err(err) = psm_c.load_series_from_file() {
                psm_c.report_error("Load file failed.", &err);
            }
        });

        psm
    }

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
        };
        Ok(())
    }

    pub fn connect_add_paint<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(&self, callback: F) {
        self.binder.connect_popup_menu_item("add", callback);
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.binder.set_target_rgb(rgb);
    }
}
