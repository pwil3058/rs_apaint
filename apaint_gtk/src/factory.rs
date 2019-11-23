// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk::prelude::*;

use crate::colour_edit::ColourEditor;
use crate::graticule::GtkGraticule;
use crate::list::{ColouredItemListView, PaintListHelper};

use apaint::basic_paint::{BasicPaint, BasicPaintBuilder};

use colour_math::ScalarAttribute;

use apaint::characteristics::CharacteristicType;
use apaint_gtk_boilerplate::PWO;
use pw_gix::sav_state::{ConditionalWidgetGroups, MaskedCondns, WidgetStatesControlled};
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
    pub const SAV_ID_READY: u64 = 1 << 0;
    pub const SAV_NAME_READY: u64 = 1 << 1;
    pub const SAV_NOTES_READY: u64 = 1 << 2;

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
        let add_btn = gtk::ButtonBuilder::new().label("Add").build();
        let colour_editor = ColourEditor::new(attributes, &[add_btn.clone()]);
        vbox.pack_start(&colour_editor.pwo(), true, true, 0);
        let buttons = ConditionalWidgetGroups::<gtk::Button>::new(
            WidgetStatesControlled::Sensitivity,
            None,
            None,
        );
        buttons.add_widget("add", &add_btn, Self::SAV_ID_READY);
        let bpe = Rc::new(Self {
            vbox,
            id_entry,
            name_entry,
            notes_entry,
            colour_editor,
            buttons,
        });

        let bpe_c = Rc::clone(&bpe);
        add_btn.connect_clicked(move |_| bpe_c.process_add_action());

        let bpe_c = Rc::clone(&bpe);
        bpe.id_entry.connect_changed(move |entry| {
            if entry.get_text_length() > 0 {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: Self::SAV_ID_READY,
                    mask: Self::SAV_ID_READY,
                })
            } else {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: 0,
                    mask: Self::SAV_ID_READY,
                })
            }
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.name_entry.connect_changed(move |entry| {
            if entry.get_text_length() > 0 {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: Self::SAV_NAME_READY,
                    mask: Self::SAV_NAME_READY,
                })
            } else {
                bpe_c.buttons.update_condns(MaskedCondns {
                    condns: 0,
                    mask: Self::SAV_NAME_READY,
                })
            }
        });

        let bpe_c = Rc::clone(&bpe);
        bpe.notes_entry.connect_changed(move |entry| {
            let condns = if entry.get_text_length() > 0 {
                Self::SAV_NOTES_READY
            } else {
                0
            };
            bpe_c.buttons.update_condns(MaskedCondns {
                condns,
                mask: Self::SAV_NOTES_READY,
            });
        });

        bpe
    }

    fn process_add_action(&self) {
        let id = self
            .id_entry
            .get_text()
            .expect("shouldn't be called otherwise");
        let rgb = self.colour_editor.rgb();
        let mut paint_builder = BasicPaintBuilder::new(&id, &rgb);
        if let Some(name) = self.name_entry.get_text() {
            paint_builder = paint_builder.name(&name);
        }
        if let Some(notes) = self.notes_entry.get_text() {
            paint_builder = paint_builder.notes(&notes);
        }
    }
}

#[derive(PWO)]
pub struct BasicPaintFactory {
    _paned: gtk::Paned,
    _paint_editor: Rc<BasicPaintEditor>,
    _hue_wheel: Rc<GtkGraticule>,
    _list_view: Rc<ColouredItemListView>,
    _paint_list_helper: PaintListHelper,
}

impl BasicPaintFactory {
    pub fn new(attributes: &[ScalarAttribute], characteristics: &[CharacteristicType]) -> Rc<Self> {
        let _paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        let _paint_editor = BasicPaintEditor::new(attributes, &[]);
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
