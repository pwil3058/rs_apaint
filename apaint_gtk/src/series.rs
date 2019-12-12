// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use gtk::prelude::*;

use pw_gix::{
    gtkx::{
        notebook::{TabRemoveLabel, TabRemoveLabelInterface},
        window::RememberGeometry,
    },
    sav_state::{MaskedCondns, SAV_HOVER_OK},
    wrapper::*,
};

use apaint_gtk_boilerplate::{Wrapper, PWO};

use apaint::{
    characteristics::CharacteristicType,
    hue_wheel::MakeColouredShape,
    series::{SeriesPaint, SeriesPaintSeries},
    spec::SeriesId,
};

use crate::{
    colour::{ScalarAttribute, RGB},
    hue_wheel::GtkHueWheel,
    icon_image::series_paint_image,
    list::{ColouredItemListView, PaintListHelper, PaintListRow},
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
        let list_helper = PaintListHelper::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(
            &list_helper.column_types(),
            &list_helper.columns(),
            menu_items,
        );
        for paint in paint_series.paints() {
            hue_wheel.add_item(paint.coloured_shape());
            let row = paint.row(&list_helper);
            //list_helper.rc_row(paint);
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
        for page in self.pages.borrow().iter() {
            page.set_target_rgb(rgb);
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
                self.pages.borrow_mut().insert(index, new_page);
                Ok(())
            }
        }
    }
}

pub struct PaintSeriesManagerWindow {
    window: gtk::Window,
    binder: Rc<SeriesBinder>,
    is_iconified: Cell<bool>,
}

impl PaintSeriesManagerWindow {
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
        let window = gtk::WindowBuilder::new()
            .destroy_with_parent(true)
            .title("SeriesPaintManager")
            .build();
        window.set_geometry_from_recollections("series_paint_manager", (200, 300));
        window.connect_delete_event(move |w, _| {
            w.hide_on_delete();
            gtk::Inhibit(true)
        });
        window.add(&binder.pwo());

        let psmw = Rc::new(Self {
            window,
            binder,
            is_iconified: Cell::new(false),
        });

        let psmw_c = Rc::clone(&psmw);
        psmw.window.connect_window_state_event(move |_, event| {
            let state = event.get_new_window_state();
            let is_iconified = state.contains(gdk::WindowState::ICONIFIED);
            psmw_c.is_iconified.set(is_iconified);
            gtk::Inhibit(false)
        });

        psmw
    }

    pub fn connect_add_paint<F: Fn(Rc<SeriesPaint<f64>>) + 'static>(&self, callback: F) {
        self.binder.connect_popup_menu_item("add", callback);
    }

    pub fn set_target_rgb(&self, rgb: Option<&RGB>) {
        self.binder.set_target_rgb(rgb);
    }
}

pub trait WindowPresentButton {
    fn window_present_button(&self) -> gtk::Button;
}

impl WindowPresentButton for Rc<PaintSeriesManagerWindow> {
    fn window_present_button(&self) -> gtk::Button {
        let button = gtk::ButtonBuilder::new()
            .image(&series_paint_image(24).upcast::<gtk::Widget>())
            .tooltip_text("Open/raise Paint Series Manager.")
            .build();
        let self_c = Rc::clone(self);
        button.connect_clicked(move |_| {
            // NB: diconify() is unreliable due to window manager interference
            if self_c.window.get_visible() && !self_c.is_iconified.get() {
                self_c.window.hide();
            } else {
                self_c.window.present();
            }
        });
        button
    }
}
