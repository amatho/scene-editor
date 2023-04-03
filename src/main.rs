use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    scene_editor::run()?;

    Ok(())
}
