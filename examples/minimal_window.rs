use oxide::prelude::*;

struct MinimalWorkspace;

impl AppHandler for MinimalWorkspace {
    fn on_render(&mut self, ctx: &mut Context) {
        ctx.clear(Color::rgb(0.1, 0.12, 0.16));
    }
}

fn main() -> Result<(), OxideError> {
    App::build()
        .with_title("Oxide - Minimal Window")
        .with_size(800, 600)
        .run(MinimalWorkspace)
}
