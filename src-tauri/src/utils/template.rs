use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json;
use serde_with::{serde_as, NoneAsEmptyString};
use tokio::{fs, io::AsyncWriteExt};
use ts_rs::TS;

use crate::ProcessError;

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
pub struct LowerThird {
    pub path: String,
    pub duration: f64,
    pub position: Vec<String>,
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
pub struct Template {
    #[ts(type = "string")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub intro: Option<String>,
    #[serde(default)]
    pub intro_duration: f64,
    #[ts(type = "string")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub outro: Option<String>,
    #[serde(default)]
    pub outro_duration: f64,
    pub lower_thirds: Vec<LowerThird>,
}

impl Template {
    pub async fn new(path: &Path) -> Result<Self, ProcessError> {
        let contents = fs::read_to_string(path).await?;
        let template: Self = serde_json::from_str(&contents)?;

        Ok(template)
    }

    pub async fn save(self, path: &str) -> Result<(), ProcessError> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = fs::File::create(path).await?;
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }
}
