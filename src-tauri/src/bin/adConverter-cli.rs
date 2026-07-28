use std::process::exit;

use adconverter_lib::{cli, init_logging};
use log::error;

#[tokio::main]
async fn main() {
    let _logger = init_logging(None);

    if let Err(error) = cli::encoder::run().await {
        error!("{error}");
        exit(1);
    }
}
