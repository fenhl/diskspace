#![cfg(windows)]

#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        collections::BTreeMap,
        process::Command,
        sync::Arc,
        time::Duration as UDuration,
    },
    bytesize::ByteSize,
    native_windows_derive as nwd,
    native_windows_gui::{
        self as nwg,
        NativeUi as _,
    },
    parking_lot::Mutex,
    systemstat::{
        Filesystem,
        Platform as _,
        System,
    },
    tokio::{
        runtime::Runtime,
        time::sleep,
    },
    wheel::traits::CommandExt as _,
};

#[derive(Default, nwd::NwgUi)]
pub struct SystemTray {
    runtime: Option<Runtime>,
    volumes: Arc<Mutex<BTreeMap<&'static str, Filesystem>>>,
    #[nwg_control]
    #[nwg_events(OnInit: [SystemTray::init])]
    window: nwg::MessageWindow,
    #[nwg_control]
    #[nwg_events(OnNotice: [SystemTray::set_icon])]
    update_notice: nwg::Notice,
    #[nwg_resource(source_file: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo-black-16.ico")))]
    logo_black_16: nwg::Icon,
    #[nwg_resource(source_file: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo-black-32.ico")))]
    logo_black_32: nwg::Icon,
    #[nwg_resource(source_file: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo-white-16.ico")))]
    logo_white_16: nwg::Icon,
    #[nwg_resource(source_file: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo-white-32.ico")))]
    logo_white_32: nwg::Icon,
    #[nwg_control(icon: Some(&data.logo_white_16), tip: Some("checking disk space…"))]
    #[nwg_events(MousePressLeftUp: [SystemTray::open_windirstat], OnContextMenu: [SystemTray::show_menu])]
    tray: nwg::TrayNotification,
    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,
    #[nwg_control(parent: tray_menu, text: "Cargo Sweep")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::cargo_sweep])]
    item_cargo_sweep: nwg::MenuItem,
    #[nwg_control(parent: tray_menu)]
    sep: nwg::MenuSeparator,
    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::exit])]
    item_exit: nwg::MenuItem,
}

impl SystemTray {
    fn init(&self) {
        self.set_icon();
        self.runtime.as_ref().expect("failed to create tokio runtime").spawn(maintain(Arc::clone(&self.volumes), self.update_notice.sender()));
    }

    fn cargo_sweep(&self) {
        Command::new("wt")
            .arg("new-tab")
            .arg("C:\\Program Files\\PowerShell\\7\\pwsh.exe")
            .arg("-c")
            .arg("cd C:\\Users\\fenhl\\git && cargo sweep -ir")
            .create_no_window()
            .spawn().expect("failed to open Windows Terminal");
        Command::new("wt")
            .arg("new-tab")
            .arg("--profile")
            .arg("debian-m2")
            .arg("C:\\WINDOWS\\system32\\wsl.exe")
            .arg("-d")
            .arg("debian-m2")
            .arg("zsh")
            .arg("-c")
            .arg("cd wslgit && cargo sweep -ir && cd /opt/git && cargo sweep -ir")
            .create_no_window()
            .spawn().expect("failed to open Windows Terminal");
        Command::new("wt")
            .arg("new-tab")
            .arg("--profile")
            .arg("ubuntu-m2")
            .arg("C:\\WINDOWS\\system32\\wsl.exe")
            .arg("-d")
            .arg("ubuntu-m2")
            .arg("zsh")
            .arg("-c")
            .arg("cd wslgit && cargo sweep -ir && cd /opt/git && cargo sweep -ir")
            .create_no_window()
            .spawn().expect("failed to open Windows Terminal");
    }

    fn set_icon(&self) {
        let is_light = registry::Hive::CurrentUser.open(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", registry::Security::QueryValue).ok()
            .and_then(|key| key.value("SystemUsesLightTheme").ok())
            .map_or(false, |data| matches!(data, registry::Data::U32(1)));
        let volumes = self.volumes.lock();
        self.tray.set_visibility(!volumes.is_empty());
        self.tray.set_icon(match (is_light, nwg::scale_factor() >= 1.5) {
            (true, true) => &self.logo_black_32,
            (true, false) => &self.logo_black_16,
            (false, true) => &self.logo_white_32,
            (false, false) => &self.logo_white_16,
        });
        self.tray.set_tip(&volumes.iter()
            .min_by_key(|(_, fs)| fs.avail)
            .map(|(vol, fs)| format!("{vol}: {}% ({})", (100.0 * fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) as u8, fs.avail.to_string_as(true)))
            .unwrap_or_default()
        );
    }

    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn open_windirstat(&self) {
        Command::new("C:\\Users\\fenhl\\scoop\\shims\\windirstat.exe").create_no_window().spawn().expect("failed to open WinDirStat");
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)] Io(#[from] tokio::io::Error),
}

fn check() -> Result<BTreeMap<&'static str, Filesystem>, Error> {
    let sys = System::new();
    let volumes = ["C:\\", "D:\\", "E:\\"].into_iter()
        .map(|vol| sys.mount_at(vol).map(|fs| (vol, fs)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    Ok(if volumes.iter().any(|(_, fs)| fs.avail < ByteSize::gib(5) || (fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) < 0.05) {
        volumes
    } else {
        BTreeMap::default()
    })
}

async fn maintain_inner(volumes: Arc<Mutex<BTreeMap<&'static str, Filesystem>>>, update_notifier: nwg::NoticeSender) -> Result<(), Error> {
    loop {
        let new_volumes = check()?;
        *volumes.lock() = new_volumes;
        update_notifier.notice();
        sleep(UDuration::from_secs(2 * 60)).await;
    }
}

async fn maintain(volumes: Arc<Mutex<BTreeMap<&'static str, Filesystem>>>, update_notifier: nwg::NoticeSender) {
    if let Err(e) = maintain_inner(volumes, update_notifier).await {
        nwg::fatal_message(concat!(env!("CARGO_PKG_NAME"), ": fatal error"), &format!("{e}\nDebug info: ctx = maintain, {e:?}"))
    }
}

fn gui_main() -> Result<(), nwg::NwgError> {
    nwg::init()?;
    let app = SystemTray::build_ui(SystemTray {
        runtime: Runtime::new().ok(),
        ..SystemTray::default()
    })?;
    nwg::dispatch_thread_events();
    drop(app);
    Ok(())
}

fn main() {
    if let Err(e) = gui_main() {
        nwg::fatal_message(concat!(env!("CARGO_PKG_NAME"), ": fatal error"), &format!("{e}\nDebug info: ctx = main, {e:?}"))
    }
}
