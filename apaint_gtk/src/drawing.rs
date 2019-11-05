// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk::WidgetExt;

use apaint_gtk_boilerplate::{Wrapper, PWO};
use pw_gix::wrapper::*;

pub use apaint::drawing::*;

use crate::colour::RGB;

#[derive(PWO, Wrapper)]
pub struct Drawer {
    pub drawing_area: gtk::DrawingArea,
    pub cairo_context: cairo::Context,
}

impl Draw<f64> for Drawer {
    fn size(&self) -> Size<f64> {
        Size {
            width: self.drawing_area.get_allocated_width() as f64,
            height: self.drawing_area.get_allocated_height() as f64,
        }
    }

    fn draw_circle(&self, _centre: Point<f64>, _radius: f64, _filled: bool) {}
    fn draw_diamond(&self, _centre: Point<f64>, _side_length: f64, _filled: bool) {}
    fn draw_line(&self, _line: &[Point<f64>]) {}
    fn draw_polygon(&self, _polygon: &[Point<f64>], _filled: bool) {}
    fn draw_square(&self, _centre: Point<f64>, _side_length: f64, _filled: bool) {}
    fn draw_isosceles(&self, _position: Point<f64>, _dirn: Dirn, _size: f64, _filled: bool) {}
    fn draw_text(&self, _text: &str, _position: TextPosn<f64>, _font_size: f64, _colour: RGB) {}
    fn move_to_point(&self, _point: Point<f64>) {}
    fn line_to_point(&self, _point: Point<f64>) {}
    fn set_line_width(&self, _width: f64) {}
    fn set_line_colour(&self, _rgb: RGB) {}
    fn set_fill_colour(&self, _rgb: RGB) {}
    fn paint_linear_gradient(
        &self,
        _posn: Point<f64>,
        _size: Size<f64>,
        _colour_stops: &[(RGB, f64)],
    ) {
    }
}
