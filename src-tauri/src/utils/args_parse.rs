use std::path::PathBuf;

use clap::Parser;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
}

#[derive(Parser, Debug, Clone)]
#[clap(version,
    about = "Presets based batch converter.",
    long_about = None,
next_line_help = false,
)]
pub struct Args {
    #[clap(
        short,
        long,
        help = "Path to the presets, an empty value will fall back to the directory next to the binary."
    )]
    pub presets: Option<PathBuf>,
}
