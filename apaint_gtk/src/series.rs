// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gtk::prelude::*;

use pw_gix::{
    gtkx::notebook::{TabRemoveLabel, TabRemoveLabelInterface},
    wrapper::*,
};

use apaint_gtk_boilerplate::{Wrapper, PWO};

use apaint::{
    characteristics::CharacteristicType,
    hue_wheel::MakeColouredShape,
    series::{PaintSeries, SeriesId},
    BasicPaintIfce, FromSpec,
};

use crate::managed_menu::MenuItemSpec;
use crate::{
    colour::{ScalarAttribute, RGB},
    hue_wheel::GtkHueWheel,
    list::{ColouredItemListView, PaintListHelper},
};

#[derive(PWO, Wrapper)]
pub struct SeriesPage<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    paned: gtk::Paned,
    paint_series: PaintSeries<f64, P>,
    hue_wheel: Rc<GtkHueWheel>,
    callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(&SeriesId, &P)>>>>,
}

impl<P> SeriesPage<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    pub fn new(
        paint_series: PaintSeries<f64, P>,
        menu_items: &[MenuItemSpec],
        attributes: &[ScalarAttribute],
        characteristics: &[CharacteristicType],
    ) -> Rc<Self> {
        let paned = gtk::PanedBuilder::new().build();
        let hue_wheel = GtkHueWheel::new(menu_items, attributes);
        let list_helper = PaintListHelper::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(
            &list_helper.column_types(),
            &list_helper.columns(),
            menu_items,
        );
        for paint in paint_series.paints() {
            hue_wheel.add_item(paint.coloured_shape());
            let row = list_helper.row(paint);
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
            list_view.connect_popup_menu_item(menu_item.name(), move |id| {
                sp_c.invoke_named_callback(&item_name_c, id)
            });
            sp.callbacks
                .borrow_mut()
                .insert(menu_item.name().to_string(), vec![]);
        }

        sp
    }

    pub fn series_id(&self) -> &SeriesId {
        self.paint_series.series_id()
    }

    pub fn connect_popup_menu_item<F: Fn(&SeriesId, &P) + 'static>(&self, name: &str, callback: F) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    pub fn invoke_named_callback(&self, item: &str, id: &str) {
        if let Some(paint) = self.paint_series.find(id) {
            let sid = self.paint_series.series_id();
            for callback in self
                .callbacks
                .borrow()
                .get(item)
                .expect("invalid name")
                .iter()
            {
                callback(sid, paint)
            }
        }
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.hue_wheel.set_target_rgb(rgb);
    }
}

#[derive(PWO, Wrapper)]
pub struct SeriesBinder<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    notebook: gtk::Notebook,
    pages: RefCell<Vec<Rc<SeriesPage<P>>>>,
    menu_items: Vec<MenuItemSpec>,
    attributes: Vec<ScalarAttribute>,
    characteristics: Vec<CharacteristicType>,
    callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(&SeriesId, &P)>>>>,
}

impl<P> SeriesBinder<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    pub fn new(
        menu_items: &[MenuItemSpec],
        attributes: &[ScalarAttribute],
        characteristics: &[CharacteristicType],
    ) -> Rc<Self> {
        let notebook = gtk::NotebookBuilder::new().build();
        let pages = RefCell::new(vec![]);
        let mut hash_map: HashMap<String, Vec<Box<dyn Fn(&SeriesId, &P)>>> = HashMap::new();
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
            callbacks,
        })
    }

    fn binary_search_series_id(&self, sid: &SeriesId) -> Result<usize, usize> {
        self.pages
            .borrow()
            .binary_search_by_key(&sid, |page| page.series_id())
    }

    pub fn connect_popup_menu_item<F: Fn(&SeriesId, &P) + 'static>(&self, name: &str, callback: F) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    pub fn invoke_named_callback(&self, item: &str, sid: &SeriesId, paint: &P) {
        for callback in self
            .callbacks
            .borrow()
            .get(item)
            .expect("invalid name")
            .iter()
        {
            callback(sid, paint)
        }
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        for page in self.pages.borrow().iter() {
            page.set_target_rgb(rgb);
        }
    }
}

pub trait RcSeriesBinder<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    fn add_series(&self, new_series: PaintSeries<f64, P>) -> Result<(), crate::Error>;
}

impl<P> RcSeriesBinder<P> for Rc<SeriesBinder<P>>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    fn add_series(&self, new_series: PaintSeries<f64, P>) -> Result<(), crate::Error> {
        match self.binary_search_series_id(new_series.series_id()) {
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
                // TODO: make connections for page removal
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
                for menu_item in self.menu_items.iter() {
                    let self_c = Rc::clone(self);
                    let item_name_c = menu_item.name().to_string();
                    new_page.connect_popup_menu_item(menu_item.name(), move |sid, paint| {
                        self_c.invoke_named_callback(&item_name_c, sid, paint)
                    });
                }
                self.notebook.insert_page_menu(
                    &new_page.pwo(),
                    Some(&label.pwo()),
                    Some(&menu_label),
                    Some(index as u32),
                );
                self.pages.borrow_mut().insert(index, new_page);
                Ok(())
            }
        }
    }
}