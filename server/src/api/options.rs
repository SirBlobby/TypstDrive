use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::compile::render::PageImage;

const SMALLEST_SCALE: f32 = 0.25;
const LARGEST_SCALE: f32 = 4.0;
const LONGEST_LENGTH: f32 = 2000.0;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderFormat {
    Png,
    Pdf,
    Svg,
    Html,
}

pub const PAGED_FORMATS: &[RenderFormat] = &[RenderFormat::Pdf, RenderFormat::Png, RenderFormat::Svg];
pub const ALL_FORMATS: &[RenderFormat] = &[RenderFormat::Pdf, RenderFormat::Png, RenderFormat::Svg, RenderFormat::Html];

impl RenderFormat {
    fn from_name(value: &str) -> Option<Self> {
        match value {
            "png" => Some(Self::Png),
            "pdf" => Some(Self::Pdf),
            "svg" => Some(Self::Svg),
            "html" => Some(Self::Html),
            _ => None,
        }
    }

    pub fn choose(value: Option<&str>, default: RenderFormat, allowed: &[RenderFormat]) -> Result<Self, String> {
        let format = match value {
            None => default,
            Some(name) => Self::from_name(name).filter(|format| allowed.contains(format)).ok_or_else(|| {
                let names = allowed.iter().map(|format| format!("'{}'", format.as_str())).collect::<Vec<_>>().join(", ");
                format!("Invalid format. Must be one of {names}.")
            })?,
        };
        Ok(format)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Pdf => "pdf",
            Self::Svg => "svg",
            Self::Html => "html",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Pdf => "application/pdf",
            Self::Svg => "image/svg+xml",
            Self::Html => "text/html; charset=utf-8",
        }
    }
}

pub fn page_image(page: Option<usize>, scale: Option<f32>) -> Result<PageImage, String> {
    let page = page.unwrap_or(1);
    if page == 0 {
        return Err("page starts at 1.".to_string());
    }
    let scale = scale.unwrap_or(1.0);
    if !(SMALLEST_SCALE..=LARGEST_SCALE).contains(&scale) {
        return Err(format!("scale must be between {SMALLEST_SCALE} and {LARGEST_SCALE}."));
    }
    Ok(PageImage { page, scale })
}

pub fn check_length(length: &str, units: &[&str]) -> Result<(), String> {
    let number = units.iter().find_map(|unit| length.strip_suffix(unit));
    match number.and_then(|number| number.parse::<f32>().ok()) {
        Some(value) if value > 0.0 && value <= LONGEST_LENGTH => Ok(()),
        _ => Err(format!("width must be a positive length using one of these units: {}.", units.join(", "))),
    }
}

pub fn bad_request(message: impl Into<String>) -> Response {
    (StatusCode::BAD_REQUEST, message.into()).into_response()
}

pub fn content_response(content_type: &'static str, data: Vec<u8>) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], data).into_response()
}
