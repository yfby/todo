mod app;
mod task;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| app::App::default().run(terminal))
}
