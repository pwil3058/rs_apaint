// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use pw_gix::sav_state::{ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled};
use pw_gix::wrapper::*;

use colour_math::ScalarAttribute;

use apaint::{
    basic_paint::{BasicPaint, BasicPaintBuilder},
    characteristics::CharacteristicType,
    BasicPaintSpec,
};

use apaint_gtk_boilerplate::PWO;

use crate::colour_edit::ColourEditor;
use crate::graticule::GtkGraticule;
use crate::list::{ColouredItemListView, PaintListHelper};
use crate::spec_edit::BasicPaintSpecEditor;

#[derive(PWO)]
pub struct BasicPaintFactory {
    _paned: gtk::Paned,
    _paint_editor: Rc<BasicPaintSpecEditor>,
    _hue_wheel: Rc<GtkGraticule>,
    _list_view: Rc<ColouredItemListView>,
    _paint_list_helper: PaintListHelper,
}

impl BasicPaintFactory {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let _paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        let _paint_editor = BasicPaintSpecEditor::new(attributes, &[]);
        let _hue_wheel = GtkGraticule::new(&[], attributes);
        let _paint_list_helper = PaintListHelper::new(attributes, characteristics);
        let list_store = _paint_list_helper.new_list_store(&[]);
        let _list_view = ColouredItemListView::new(&list_store, &_paint_list_helper.columns(), &[]);
        Rc::new(Self {
            _paned,
            _paint_editor,
            _hue_wheel,
            _list_view,
            _paint_list_helper,
        })
    }
}
