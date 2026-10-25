pub mod handlers;
pub mod marking;
pub mod policy;
pub mod purge;

use axum::http::StatusCode;

pub use marking::move_to_trash;

pub type ApiError = (StatusCode, String);

fn internal_error(error: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrashKind {
    Document,
    Project,
    File,
    Folder,
}

impl TrashKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Project => "project",
            Self::File => "file",
            Self::Folder => "folder",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "document" => Some(Self::Document),
            "project" => Some(Self::Project),
            "file" => Some(Self::File),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }
}
