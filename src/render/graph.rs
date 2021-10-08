use crate::checks;

pub struct Graph {
    surface: cairo::ImageSurface,
    ctx: cairo::Context,
}

impl Graph {
    pub fn new() -> Result<Graph, cairo::Error> {
        let surface = cairo::ImageSurface::create(
            cairo::Format::Rgb24,
            100,
            100
        )?;
        let ctx = cairo::Context::new(&surface)?;
        Ok(Graph {surface, ctx})
    }
}
