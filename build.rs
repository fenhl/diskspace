use {
    std::{
        env,
        io,
    },
    winresource::WindowsResource,
};

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        println!("cargo:rerun-if-changed=assets/manifest.xml");
        WindowsResource::new()
            //TODO icon?
            .set_manifest_file("assets/manifest.xml")
            .set_icon_with_id("assets/logo-black-16.ico", "LOGO_BLACK_16")
            .set_icon_with_id("assets/logo-black-32.ico", "LOGO_BLACK_32")
            .set_icon_with_id("assets/logo-white-16.ico", "LOGO_WHITE_16")
            .set_icon_with_id("assets/logo-white-32.ico", "LOGO_WHITE_32")
            .compile()?;
    }
    Ok(())
}
