use std::{process::exit, time::SystemTimeError};

use old_files::*;

fn main() -> Result<(), SystemTimeError> {
    let config = match Config::new() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("{}",err);
            exit(1);
        }
    };

    crawl(&config)?;
    Ok(())
}
