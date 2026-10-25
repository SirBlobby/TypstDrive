use crate::AppState;

const DEFAULT_ACCENT: &str = "#239dad";

pub struct EmailBrand {
    pub site_name: String,
    pub accent: String,
    pub logo_url: Option<String>,
}

impl Default for EmailBrand {
    fn default() -> Self {
        Self { site_name: "TypstDrive".to_string(), accent: DEFAULT_ACCENT.to_string(), logo_url: None }
    }
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|character| character.is_ascii_hexdigit())
}

pub async fn load(state: &AppState) -> EmailBrand {
    let settings = crate::admin::branding::load(state).await;
    let accent = if is_hex_color(&settings.accent_color) { settings.accent_color } else { DEFAULT_ACCENT.to_string() };
    let logo_url = match crate::admin::branding::logo_updated_at(state).await {
        Some(updated_at) => crate::site_url::link(&format!("/api/branding/logo?v={}", updated_at.replace(' ', "T"))),
        None => None,
    };
    EmailBrand { site_name: settings.site_name, accent, logo_url }
}
