#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        io,
        path::PathBuf,
        process::exit,
    },
    bytesize::ByteSize,
    structopt::StructOpt,
    systemstat::{
        Platform,
        System,
    },
};

#[derive(StructOpt)]
struct Args {
    /// Produce no output unless an error occurs during the calculation. Exit status will be 1 if less than --min-percent or --min-space available.
    #[structopt(short, long)]
    quiet: bool,
    /// Produce more detailed output
    #[structopt(short, long)]
    verbose: bool,
    /// Print the raw number of bytes instead of a human-readable format. Ignored if --verbose is given.
    #[structopt(long)]
    bytes: bool,
    /// Print the raw number of available files instead of a human-readable format. Ignored if --verbose or --bytes is given.
    #[structopt(long)]
    files: bool,
    /// Produce no output if at least MIN_PERCENT% of disk space is available.
    #[structopt(long)]
    min_percent: Option<f64>,
    /// Produce no output if at least MIN_SPACE GB is available.
    #[structopt(long)]
    min_space: Option<u64>,
    /// Produce no output if at least MIN_FILES_PERCENT% of files are available.
    #[structopt(long)]
    min_files_percent: Option<f64>,
    /// Produce no output if at least MIN_FILES files are available.
    #[structopt(long)]
    min_files: Option<usize>,
    /// Defaults for using in the zsh right prompt, equivalent to --min-percent=5 --min-space=5 --min-files=5000.
    #[structopt(long)]
    zsh: bool,
    /// The mount path of the partition whose space to check.
    #[structopt(parse(from_os_str), default_value = "/")]
    path: PathBuf,
}

#[wheel::main]
fn main(args: Args) -> io::Result<()> {
    let min_fraction = if let Some(min_percent) = args.min_percent { min_percent / 100.0 } else if args.zsh { 0.05 } else if args.min_space.is_some() { 0.0 } else { 1.0 };
    let min_space = if let Some(min_space) = args.min_space { ByteSize::gib(min_space) } else if args.zsh { ByteSize::gib(5) } else if args.min_percent.is_some() { ByteSize::b(0) } else { ByteSize::b(u64::MAX) };
    let min_files_fraction = if let Some(min_files_percent) = args.min_files_percent { min_files_percent / 100.0 } else if args.zsh { 0.05 } else if args.min_files.is_some() { 0.0 } else { 1.0 };
    let min_files = if let Some(min_files) = args.min_files { min_files } else if args.zsh { 5000 } else if args.min_files_percent.is_some() { 0 } else { usize::MAX };
    let fs = System::new().mount_at(args.path)?;
    if fs.avail < min_space || (fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) < min_fraction
    || fs.files_avail < min_files || (fs.files_avail as f64 / fs.files_total as f64) < min_files_fraction {
        if args.quiet {
            exit(1);
        } else if args.verbose {
            println!("Available disk space: {}", fs.avail);
            println!("{} bytes free", fs.avail.as_u64());
            println!("{} bytes total", fs.total.as_u64());
            println!("{} percent", (100.0 * fs.avail.as_u64() as f64 / fs.total.as_u64() as f64) as u8);
            println!("{} files free", fs.files_avail);
            println!("{} files total", fs.files_total);
            println!("{} percent", (100.0 * fs.files_avail as f64 / fs.files_total as f64) as u8);
        } else if args.bytes {
            println!("{}", fs.avail.as_u64());
        } else if args.files {
            println!("{}", fs.files_avail);
        } else if fs.files_avail < min_files || (fs.files_avail as f64 / fs.files_total as f64) < min_files_fraction {
            println!("[disk: {} ({} files)]", fs.avail, fs.files_avail);
        } else {
            println!("[disk: {}]", fs.avail);
        }
    }
    Ok(())
}
