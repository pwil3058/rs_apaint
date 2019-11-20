// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use crate::colour_edit::ColourEditor;
use crate::graticule::GtkGraticule;
use crate::list::{ColouredItemListView, PaintListHelper};

use apaint::basic_paint::BasicPaint;

use colour_math::ScalarAttribute;

use apaint::characteristics::CharacteristicType;
use apaint_gtk_boilerplate::PWO;
use gdk::enums::key::ch;
use pw_gix::wrapper::*;

#[derive(PWO)]
pub struct BasicPaintFactory {
    _paned: gtk::Paned,
    _colour_editor: Rc<ColourEditor>,
    _hue_wheel: Rc<GtkGraticule<BasicPaint<f64>>>,
    _list_view: Rc<ColouredItemListView>,
    _paint_list_helper: PaintListHelper,
}

impl BasicPaintFactory {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let _paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        let _colour_editor = ColourEditor::new(attributes, &[]);
        let _hue_wheel = GtkGraticule::<BasicPaint<f64>>::new(&[], attributes);
        let _paint_list_helper = PaintListHelper::new(attributes, characteristics);
        let list_store = _paint_list_helper.new_list_store(&[]);
        let _list_view = ColouredItemListView::new(&list_store, &_paint_list_helper.columns(), &[]);
        Rc::new(Self {
            _paned,
            _colour_editor,
            _hue_wheel,
            _list_view,
            _paint_list_helper,
        })
    }
}
