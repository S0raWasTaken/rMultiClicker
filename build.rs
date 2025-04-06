use std::{env::var_os, io};

use winresource::WindowsResource;

fn main() -> io::Result<()> {
    if var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("rmulticlicker.ico")
            .compile()?;
    }
    Ok(())
}
