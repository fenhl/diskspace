use std::{
    collections::BTreeMap,
    fs::File,
    io,
    path::{
        Path,
        PathBuf
    }
};
use bytesize::ByteSize;
use serde_derive::Deserialize;
use systemstat::{
    Platform,
    System
};
use wrapped_enum::wrapped_enum;
use bitbar::{
    ContentItem,
    Menu,
    MenuItem
};

/// hdd-o from Fork Awesome, see LICENSE
const DISK_ICON: &str = "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";

wrapped_enum! {
    #[derive(Debug)]
    enum Error {
        Io(io::Error),
        Json(serde_json::Error)
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
    Ok(if volumes.iter().any(|(_, fs)| fs.avail < ByteSize::gib(5) || (fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) < 0.05) {
        vec![
            ContentItem::new(volumes.iter().map(|(_, fs)| fs.avail).min().expect("no volumes")).template_image(DISK_ICON).into(),
            MenuItem::Sep,
        ].into_iter().chain(
            volumes.into_iter().map(|(vol, fs)| MenuItem::new(format!("{}: {}% ({})", vol.display(), (100.0 * fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) as u8, fs.avail)))
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
                ContentItem::new("?").template_image(DISK_ICON).into(),
                MenuItem::Sep,
                MenuItem::new(format!("{:?}", e))
            ]));
        }
    }
}
