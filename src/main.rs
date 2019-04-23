#[macro_use] extern crate clap;

use std::{
    convert::Infallible,
    io,
    num::{
        ParseFloatError,
        ParseIntError
    },
    path::Path,
    process::exit
};
use bytesize::ByteSize;
use clap::{
    App,
    Arg
};
use systemstat::{
    Platform,
    System
};
use wrapped_enum::wrapped_enum;

wrapped_enum! {
    #[derive(Debug)]
    enum Error {
        Io(io::Error),
        ParseFloat(ParseFloatError),
        ParseInt(ParseIntError)
    }
}

impl From<Infallible> for Error {
    fn from(i: Infallible) -> Error {
        match i {}
    }
}

fn app() -> App<'static, 'static> {
    app_from_crate!()
        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .help("Produce no output unless an error occurs during the calculation. Exit status will be 1 if less than --min-percent or --min-space available."))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Produce more detailed output."))
        .arg(Arg::with_name("bytes")
            .long("bytes")
            .help("Print the raw number of bytes instead of a human-readable format."))
        .arg(Arg::with_name("min-percent")
            .long("min-percent")
            .takes_value(true)
            .value_name("MIN_PERCENT")
            .help("Produce no output if at least MIN_PERCENT% of disk space is available."))
        .arg(Arg::with_name("min-space")
            .long("min-space")
            .takes_value(true)
            .value_name("MIN_SPACE")
            .help("Produce no output if at least MIN_SPACE GB is available."))
        .arg(Arg::with_name("zsh")
            .long("zsh")
            .help("Defaults for using in the zsh right prompt, equivalent to --min-percent=5 --min-space=5."))
        .arg(Arg::with_name("PATH"))
    //TODO options (except --notify)
}

fn main() -> Result<(), Error> {
    let matches = app().get_matches();
    let min_fraction = matches.value_of("min-percent").map_or_else(
        || Ok(if matches.is_present("zsh") { 0.05 } else if matches.is_present("min-space") { 0.0 } else { 1.0 }),
        |min_percent| min_percent.parse::<f64>().map(|min_percent| min_percent / 100.0)
    )?;
    let min_space = matches.value_of("min-space").map_or_else(
        || Ok(if matches.is_present("zsh") { ByteSize::gib(5) } else if matches.is_present("min-percent") { ByteSize::b(0) } else { ByteSize::b(usize::max_value()) }),
        |min_space| min_space.parse::<usize>().map(|min_space| ByteSize::gib(min_space))
    )?;
    let path = matches.value_of("PATH").map_or(
        Ok(Path::new("/").to_owned()),
        |path| path.parse()
    )?;
    let fs = System::new().mount_at(path)?;
    if fs.avail < min_space || (fs.avail.as_usize() as f64 / fs.total.as_usize() as f64) < min_fraction {
        if matches.is_present("quiet") {
            exit(1);
        } else if matches.is_present("verbose") {
            println!("Available disk space: {}", fs.avail);
            println!("{} bytes free", fs.avail.as_usize());
            println!("{} bytes total", fs.total.as_usize());
            println!("{} percent", (100.0 * fs.avail.as_usize() as f64 / fs.total.as_usize() as f64) as u8);
        } else if matches.is_present("bytes") {
            println!("{}", fs.avail.as_usize());
        } else {
            println!("[disk: {}]", fs.avail);
        }
    }
    Ok(())
}
