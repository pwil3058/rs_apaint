// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gtk::prelude::*;

use pw_gix::wrapper::*;

use apaint_gtk_boilerplate::{Wrapper, PWO};

use apaint::{
    characteristics::CharacteristicType,
    hue_wheel::MakeColouredShape,
    series::{PaintSeries, SeriesId},
    BasicPaintIfce, FromSpec,
};

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
        menu_items: &'static [(&str, &str, Option<&gtk::Image>, &str, u64)],
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
            let item_name = menu_item.0;
            sp.hue_wheel.connect_popup_menu_item(item_name, move |id| {
                sp_c.pass_on_callback_invocation(item_name, id)
            });
            let sp_c = Rc::clone(&sp);
            list_view.connect_popup_menu_item(item_name, move |id| {
                sp_c.pass_on_callback_invocation(item_name, id)
            });
            sp.callbacks
                .borrow_mut()
                .insert(item_name.to_string(), vec![]);
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

    pub fn pass_on_callback_invocation(&self, item: &str, id: &str) {
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
    pages: RefCell<Vec<SeriesPage<P>>>,
    callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(&SeriesId, &P)>>>>,
}

impl<P> SeriesBinder<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    pub fn new() -> Rc<Self> {
        let notebook = gtk::NotebookBuilder::new().build();
        let pages = RefCell::new(vec![]);
        let callbacks = RefCell::new(HashMap::new());
        Rc::new(Self {
            notebook,
            pages,
            callbacks,
        })
    }
}
