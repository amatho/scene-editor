use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    tdt4230::run()?;

    Ok(())
}
