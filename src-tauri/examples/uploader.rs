use std::{
    cmp::min,
    io::{stdout, Write},
    path::PathBuf,
};

use clap::Parser;
use tauri_plugin_http::reqwest;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

#[derive(Parser, Debug, Clone)]
#[clap(version,
    about = "Upload videos to peertube.",
    long_about = None,
next_line_help = false,
)]
pub struct Args {
    #[clap(short, long, help = "Channel ID")]
    pub channel_id: u32,

    #[clap(short, long, help = "Video name")]
    pub name: String,

    #[clap(short, long, help = "Upload token")]
    pub token: String,

    #[clap(short, long, help = "File path")]
    pub source: PathBuf,

    #[clap(short, long, help = "Upload URL")]
    pub url: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let file = tokio::fs::File::open(&args.source).await.unwrap();
    let name = args
        .source
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let total_size = file.metadata().await.unwrap().len();
    let mut reader_stream = ReaderStream::new(file);
    let mut uploaded = 0;

    let async_stream = async_stream::stream! {
        while let Some(chunk) = reader_stream.next().await {
            if let Ok(chunk) = &chunk {
                let new = min(uploaded + (chunk.len() as u64), total_size);
                uploaded = new;
                let progress = uploaded * 100 / total_size;

                print!("\rUploading {progress}%");

                if uploaded >= total_size {
                    print!("\r{progress} done...");
                }

                stdout().flush().unwrap();
            }
            yield chunk;
        }
    };

    println!("Upload: {name}");

    let part = reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(async_stream))
        .file_name(name.clone())
        .mime_str("video/mp4")
        .unwrap();

    if let Err(e) = reqwest::Client::new()
        .post(format!("{}/api/v1/videos/upload", args.url))
        // .timeout(std::time::Duration::from_secs(600))
        .bearer_auth(args.token)
        .multipart(
            reqwest::multipart::Form::new()
                .part("videofile", part)
                .text("channelId", args.channel_id.to_string())
                .text("name", "test"),
        )
        .send()
        .await
    {
        eprint!("{e}");
    };
}
