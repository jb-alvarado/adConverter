use clap::Parser;
use inquire::{Confirm, MultiSelect, Select};

use crate::{collect_presets, Config, ProcessError};

#[derive(Parser, Debug, Clone)]
#[clap(version,
    about = "adConverter CLI",
    long_about = None)]
pub struct Args {
    #[clap(long, help = "Run adConverter in CLI mode")]
    pub cli: bool,

    #[clap(short, long, help = "Files to encode", num_args = 1..)]
    pub files: Vec<String>,

    #[clap(short, long, help = "Language to transcript")]
    pub lang: Option<String>,

    #[clap(long, help = "Fade in/out video and audio")]
    pub fade: Option<bool>,

    #[clap(long, help = "Apply loudnorm filter")]
    pub lufs: Option<bool>,

    #[clap(short, long, help = "Encoding presets", num_args = 0..)]
    pub presets: Option<Vec<String>>,
}

impl Args {
    pub async fn init(config: &Config) -> Result<Self, ProcessError> {
        let mut obj = Self::parse();
        let lang_list: Vec<String> = config
            .transcript_lang
            .iter()
            .map(|r| r.name.clone())
            .collect();

        let mut preset_list: Vec<String> = collect_presets(&None)
            .await?
            .iter()
            .map(|r| r.name.clone())
            .collect();

        preset_list.insert(0, "None".to_string());

        if obj.lang.is_none() {
            let lang = Select::new("Transcript Language:", lang_list).prompt()?;

            obj.lang = Some(lang);
        }

        if obj.fade.is_none() {
            let fade = Confirm::new("Apply fade [Y/n]:").prompt()?;

            obj.fade = Some(fade);
        }

        if obj.lufs.is_none() {
            let lufs = Confirm::new("Apply loudnorm [Y/n]:").prompt()?;

            obj.lufs = Some(lufs);
        }

        if obj.presets.is_none() {
            let presets = MultiSelect::new("Encoding presets:", preset_list).prompt()?;

            let presets = presets
                .into_iter()
                .filter(|p| p != "None")
                .collect::<Vec<String>>();

            obj.presets = Some(presets);
        }

        Ok(obj)
    }
}
