use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use ts_rs::TS;

pub mod peertube;

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
pub struct Platform {
    pub name: String,
    pub username: String,
    pub password: String,
}
