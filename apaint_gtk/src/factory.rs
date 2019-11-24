// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;
use pw_gix::wrapper::*;

use colour_math::ScalarAttribute;

use apaint::{basic_paint::BasicPaint, characteristics::CharacteristicType, BasicPaintSpec};

use apaint_gtk_boilerplate::PWO;

use crate::hue_wheel::GtkHueWheel;
use crate::list::{ColouredItemListView, PaintListHelper};
use crate::spec_edit::BasicPaintSpecEditor;

#[derive(PWO)]
pub struct BasicPaintFactory {
    paned: gtk::Paned,
    paint_editor: Rc<BasicPaintSpecEditor>,
    hue_wheel: Rc<GtkHueWheel>,
    list_view: Rc<ColouredItemListView>,
    paint_list_helper: PaintListHelper,
}

impl BasicPaintFactory {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        let paint_editor = BasicPaintSpecEditor::new(attributes, &[]);
        let hue_wheel = GtkHueWheel::new(&[], attributes);
        let paint_list_helper = PaintListHelper::new(attributes, characteristics);
        let list_view = ColouredItemListView::new(
            &paint_list_helper.column_types(),
            &paint_list_helper.columns(),
            &[],
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
        });

        let bpf_c = Rc::clone(&bpf);
        bpf.paint_editor
            .connect_add_action(move |spec| bpf_c.add_paint(spec));

        bpf
    }

    fn add_paint(&self, paint_spec: &BasicPaintSpec<f64>) {
        let paint: BasicPaint<f64> = paint_spec.into();
        self.hue_wheel.add_item((&paint).into());
        let row = self.paint_list_helper.row(&paint);
        self.list_view.add_row(&row);
    }
}
