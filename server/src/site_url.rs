pub fn public_url() -> Option<String> {
    std::env::var("PUBLIC_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
}

pub fn uses_https() -> bool {
    public_url().is_some_and(|url| url.starts_with("https://"))
}

pub fn link(path: &str) -> Option<String> {
    public_url().map(|base| format!("{base}{path}"))
}

pub fn is_safe_redirect(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//") && !path.contains('\\')
}
