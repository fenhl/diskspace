use {
    std::{
        collections::BTreeMap,
        convert::Infallible,
        fs::File,
        io,
        path::{
            Path,
            PathBuf
        }
    },
    bytesize::ByteSize,
    derive_more::From,
    serde_derive::Deserialize,
    systemstat::{
        Platform,
        System
    },
    bitbar::{
        ContentItem,
        Menu,
        MenuItem
    }
};

trait ResultNeverExt<T> {
    fn never_unwrap(self) -> T;
}

impl<T> ResultNeverExt<T> for Result<T, Infallible> {
    fn never_unwrap(self) -> T {
        match self {
            Ok(inner) => inner,
            Err(never) => match never {}
        }
    }
}

#[derive(Debug, From)]
enum Error {
    Io(io::Error),
    Json(serde_json::Error)
}

impl From<Infallible> for Error {
    fn from(never: Infallible) -> Error {
        match never {}
    }
}

#[derive(Debug, Default, Deserialize)]
struct Config {
    #[serde(default)]
    diskspace: ConfigDiskSpace
}

#[derive(Debug, Default, Deserialize)]
struct ConfigDiskSpace {
    volumes: Option<Vec<PathBuf>>
}

impl Config {
    fn new() -> Result<Config, Error> {
        let dirs = xdg_basedir::get_config_home().into_iter().chain(xdg_basedir::get_config_dirs());
        for cfg_dir in dirs {
            let path = cfg_dir.join("fenhl/syncbin.json");
            if path.exists() {
                return Ok(serde_json::from_reader(File::open(path)?)?);
            }
        }
        Ok(Config::default())
    }

    fn volumes(self) -> Vec<PathBuf> {
        self.diskspace.volumes.unwrap_or_else(|| vec![Path::new("/").to_owned()])
    }
}

fn bitbar() -> Result<Menu, Error> {
    let sys = System::new();
    let volumes = Config::new()?
        .volumes()
        .into_iter()
        .map(|vol| sys.mount_at(&vol).map(|fs| (vol, fs)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    Ok(if volumes.iter().any(|(_, fs)| fs.avail < ByteSize::gib(5) || (fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) < 0.05 || fs.files_avail < 5000 || (fs.files_avail as f64 / fs.files_total as f64) < 0.05) {
        vec![
            ContentItem::new(volumes.iter().map(|(_, fs)| fs.avail).min().expect("no volumes")).template_image(&include_bytes!("../assets/disk.png")[..])?.into(),
            MenuItem::Sep,
        ].into_iter().chain(
            volumes.into_iter().map(|(vol, fs)| MenuItem::new(format!("{}: {}% ({}, {} files)", vol.display(), (100.0 * fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) as u8, fs.avail, fs.files_avail)))
        ).chain(vec![
            MenuItem::Sep,
            ContentItem::new("Open DaisyDisk").command(["/usr/bin/open", "-a", "DaisyDisk"]).into()
        ]).collect()
    } else {
        Menu::default()
    })
}

fn main() {
    match bitbar() {
        Ok(menu) => { print!("{}", menu); }
        Err(e) => {
            print!("{}", Menu(vec![
                ContentItem::new("?").template_image(&include_bytes!("../assets/disk.png")[..]).never_unwrap().into(),
                MenuItem::Sep,
                MenuItem::new(format!("{:?}", e))
            ]));
        }
    }
}
