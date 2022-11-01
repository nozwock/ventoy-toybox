use std::io;
use winres::WindowsResource;

// https://github.com/mxre/winres/issues/33

fn main() -> io::Result<()> {
    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap() == "windows"
        && std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "gnu"
    {
        WindowsResource::new()
            .set_ar_path("x86_64-w64-mingw32-ar")
            .set_windres_path("x86_64-w64-mingw32-windres")
            .set_icon("assets/icon.ico")
            .compile()?;
    }
    Ok(())
}
