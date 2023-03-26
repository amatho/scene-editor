use log::error;

fn main() -> Result<(), &'static str> {
    if let Err(e) = tdt4230::run() {
        error!("{e}");
        return Err("failed to run program");
    }

    Ok(())
}
