fn main() -> Result<(), &'static str> {
    if let Err(e) = tdt4230::run() {
        eprintln!("{e}");
        return Err("failed to run program");
    }

    Ok(())
}
