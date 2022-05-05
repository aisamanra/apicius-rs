#![allow(dead_code, unused_variables)]
use crate::checks;

pub struct RenderConfig {}

pub struct Graph {
    surface: cairo::ImageSurface,
    ctx: cairo::Context,
}

impl Graph {
    pub fn new() -> Result<Graph, cairo::Error> {
        // there's a weird thing that's going to happen here, because
        // we don't yet know how big to make the surface. That means
        // that we're going to draw onto a big image surface but might
        // need to redraw if we run out of space
        let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, 2048, 2048)?;
        let ctx = cairo::Context::new(&surface)?;
        Ok(Graph { surface, ctx })
    }

    pub fn draw(&self, tree: &checks::BackwardTree, s: &checks::State) {}
}
