use serde::Deserialize;

use crate::api::render::InlineFile;

#[derive(Deserialize)]
pub struct DocumentRequest {
    pub code: String,
    pub format: Option<String>,
    pub files: Option<Vec<InlineFile>>,
    pub page: Option<usize>,
    pub scale: Option<f32>,
}

#[derive(Deserialize)]
pub struct MathRequest {
    pub math: String,
    pub format: Option<String>,
    pub display: Option<bool>,
    pub packages: Option<Vec<String>>,
    pub scale: Option<f32>,
}

#[derive(Deserialize)]
pub struct SnippetRequest {
    pub code: String,
    pub format: Option<String>,
    pub preamble: Option<String>,
    pub packages: Option<Vec<String>>,
    pub width: Option<String>,
    pub files: Option<Vec<InlineFile>>,
    pub scale: Option<f32>,
}

#[derive(Deserialize)]
pub struct CheckRequest {
    pub code: String,
    pub files: Option<Vec<InlineFile>>,
}
