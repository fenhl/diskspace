use {
    std::io,
    winres::WindowsResource,
};

fn main() -> io::Result<()> {
    WindowsResource::new()
        //TODO icon?
        //TODO version info?
        .set_manifest_file("assets/manifest.xml")
        //TODO embed systray icons instead of loading dynamically, see https://github.com/gabdube/native-windows-gui/tree/master/native-windows-gui/examples/embed_resources
        .compile()
}
