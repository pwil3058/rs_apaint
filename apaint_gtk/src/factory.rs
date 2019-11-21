// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use crate::colour_edit::ColourEditor;
use crate::graticule::GtkGraticule;
use crate::list::{ColouredItemListView, PaintListHelper};

use apaint::basic_paint::BasicPaint;

use colour_math::ScalarAttribute;

use apaint::characteristics::CharacteristicType;
use apaint_gtk_boilerplate::PWO;
use pw_gix::sav_state::{ConditionalWidgetGroups, WidgetStatesControlled};
use pw_gix::wrapper::*;

#[derive(PWO)]
pub struct BasicPaintEditor {
    vbox: gtk::Box,
    id_entry: gtk::Entry,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    colour_editor: Rc<ColourEditor>,
    buttons: Rc<ConditionalWidgetGroups<gtk::Button>>,
}

impl BasicPaintEditor {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let grid = gtk::GridBuilder::new().hexpand(true).build();
        vbox.pack_start(&grid, false, false, 0);
        let label = gtk::LabelBuilder::new()
            .label("Id:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 0, 1, 1);
        let id_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&id_entry, 1, 0, 1, 1);
        let label = gtk::LabelBuilder::new()
            .label("Name:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 1, 1, 1);
        let name_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&name_entry, 1, 1, 1, 1);
        let label = gtk::LabelBuilder::new()
            .label("Notes:")
            .halign(gtk::Align::End)
            .build();
        grid.attach(&label, 0, 2, 1, 1);
        let notes_entry = gtk::EntryBuilder::new().hexpand(true).build();
        grid.attach(&notes_entry, 1, 2, 1, 1);
        let colour_editor = ColourEditor::new(attributes, &[]);
        vbox.pack_start(&colour_editor.pwo(), true, true, 0);
        let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
            WidgetStatesControlled::Sensitivity,
            None,
            None,
        );
        Rc::new(Self {
            vbox,
            id_entry,
            name_entry,
            notes_entry,
            colour_editor,
            buttons,
        })
    }
}

#[derive(PWO)]
pub struct BasicPaintFactory {
    _paned: gtk::Paned,
    _paint_editor: Rc<BasicPaintEditor>,
    _hue_wheel: Rc<GtkGraticule<BasicPaint<f64>>>,
    _list_view: Rc<ColouredItemListView>,
    _paint_list_helper: PaintListHelper,
}

impl BasicPaintFactory {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let _paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        let _paint_editor = BasicPaintEditor::new(attributes, &[]);
        let _hue_wheel = GtkGraticule::<BasicPaint<f64>>::new(&[], attributes);
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
