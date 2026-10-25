use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use super::ApiError;

const MAX_NAME_LENGTH: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePalette {
    pub surface: String,
    pub surface_muted: String,
    pub surface_sunken: String,
    pub line: String,
    pub ink: String,
    pub ink_muted: String,
    pub accent: String,
    pub accent_soft: String,
    pub danger: String,
    pub success: String,
    pub editor_background: String,
    pub editor_text: String,
    pub selection: String,
    pub active_line: String,
    pub cursor: String,
    pub keyword: String,
    pub string: String,
    pub number: String,
    pub comment: String,
    pub variable: String,
    pub function: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeModes {
    pub light: ThemePalette,
    pub dark: ThemePalette,
}

#[derive(Debug, Clone, Serialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub modes: ThemeModes,
}

#[derive(Debug, Deserialize)]
pub struct ThemeInput {
    pub name: String,
    #[serde(flatten)]
    pub modes: ThemeModes,
}

fn normalize_color(value: &str) -> Option<String> {
    let color = value.trim().to_ascii_lowercase();
    let digits = color.strip_prefix('#')?;
    if !digits.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    match digits.len() {
        6 => Some(color),
        3 => Some(digits.chars().fold(String::from("#"), |mut expanded, digit| {
            expanded.push(digit);
            expanded.push(digit);
            expanded
        })),
        _ => None,
    }
}

impl ThemePalette {
    fn colors_mut(&mut self) -> [(&'static str, &mut String); 21] {
        [
            ("surface", &mut self.surface),
            ("surface_muted", &mut self.surface_muted),
            ("surface_sunken", &mut self.surface_sunken),
            ("line", &mut self.line),
            ("ink", &mut self.ink),
            ("ink_muted", &mut self.ink_muted),
            ("accent", &mut self.accent),
            ("accent_soft", &mut self.accent_soft),
            ("danger", &mut self.danger),
            ("success", &mut self.success),
            ("editor_background", &mut self.editor_background),
            ("editor_text", &mut self.editor_text),
            ("selection", &mut self.selection),
            ("active_line", &mut self.active_line),
            ("cursor", &mut self.cursor),
            ("keyword", &mut self.keyword),
            ("string", &mut self.string),
            ("number", &mut self.number),
            ("comment", &mut self.comment),
            ("variable", &mut self.variable),
            ("function", &mut self.function),
        ]
    }

    fn normalized(mut self, mode: &str) -> Result<Self, ApiError> {
        for (name, value) in self.colors_mut() {
            *value = normalize_color(value)
                .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("{mode} {name} must be a color like #1e66f5")))?;
        }
        Ok(self)
    }
}

impl ThemeInput {
    pub fn normalized(self) -> Result<Self, ApiError> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Theme name cannot be empty".to_string()));
        }
        if name.chars().count() > MAX_NAME_LENGTH || name.chars().any(char::is_control) {
            return Err((StatusCode::BAD_REQUEST, format!("Theme names must be {MAX_NAME_LENGTH} characters or fewer")));
        }
        Ok(Self {
            name,
            modes: ThemeModes {
                light: self.modes.light.normalized("Light")?,
                dark: self.modes.dark.normalized("Dark")?,
            },
        })
    }

    pub fn into_theme(self, id: String) -> Theme {
        Theme { id, name: self.name, modes: self.modes }
    }
}
