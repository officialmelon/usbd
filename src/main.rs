mod filesystem;
use clap::Parser;
use filesystem::{copy, move_file, remove};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Args {
    /// mode (remove, copy, move)
    #[arg(short, long)]
    mode: String,

    /// File/Folder to enumerate (windows: E:/example.txt, linux: /media/usb1/example.txt)
    #[arg(short, long)]
    file: String,

    /// Output directory.
    #[arg(short, long)]
    output: String,

    /// (Optional) Debug Out.
    #[arg(short, long)]
    debug: bool,

    /// (Optional) Progress Bar.
    #[arg(short, long)]
    bar: bool,
}

fn main() {
    let args = Args::parse();
    let start = std::time::Instant::now();

    match args.mode.as_str() {
        "rm" => remove(args.file, args.output, args.debug, args.bar),
        "cp" => copy(args.file, args.output, args.debug, args.bar),
        "mv" => move_file(args.file, args.output, args.debug, args.bar),
        _ => println!("Invalid mode"),
    }

    let duration = start.elapsed();
    println!("Finished in: {:?}", duration);
}
