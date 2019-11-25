// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use pw_gix::wrapper::*;

use colour_math::ScalarAttribute;

use apaint::{
    characteristics::CharacteristicType, hue_wheel::MakeColouredShape, series::PaintSeries,
    BasicPaintIfce, BasicPaintSpec, FromSpec,
};

use apaint_gtk_boilerplate::PWO;

use crate::hue_wheel::GtkHueWheel;
use crate::list::{ColouredItemListView, PaintListHelper};
use crate::spec_edit::BasicPaintSpecEditor;
use crate::SAV_HAS_CHOSEN_ITEM;

#[derive(PWO)]
pub struct BasicPaintFactory<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    paned: gtk::Paned,
    paint_editor: Rc<BasicPaintSpecEditor>,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    paint_list_helper: PaintListHelper,
    paint_series: RefCell<PaintSeries<f64, P>>,
}

impl<P> BasicPaintFactory<P>
where
    P: BasicPaintIfce<f64> + FromSpec<f64> + MakeColouredShape<f64> + Clone + 'static,
{
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let menu_items: &[(&str, &str, Option<&gtk::Image>, &str, u64)] = &[(
            "remove",
            "Remove",
            None,
            "Remove the indicated paint from the series.",
            SAV_HAS_CHOSEN_ITEM,
        )];
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
        paned.add1(&notebook);
        paned.add2(&paint_editor.pwo());
        let bpf = Rc::new(Self {
            paned,
            paint_editor,
            hue_wheel,
            list_view,
            paint_list_helper,
            paint_series: RefCell::new(PaintSeries::default()),
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
}
