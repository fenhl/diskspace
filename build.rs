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
            //TODO version info?
            .set_manifest_file("assets/manifest.xml")
            //TODO embed systray icons instead of loading dynamically, see https://github.com/gabdube/native-windows-gui/tree/master/native-windows-gui/examples/embed_resources
            .compile()?;
    }
    Ok(())
}
