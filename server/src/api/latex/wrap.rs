const MAX_PACKAGES: usize = 20;
const MAX_PACKAGE_NAME_LENGTH: usize = 50;
const WIDTH_UNITS: [&str; 6] = ["pt", "cm", "mm", "in", "em", "ex"];
const MATH_PACKAGES: [&str; 2] = ["amsmath", "amssymb"];

pub struct Wrapped {
    pub text: String,
    pub lines_before_code: usize,
}

fn valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_PACKAGE_NAME_LENGTH
        && name.chars().all(|character| character.is_ascii_alphanumeric() || character == '-')
}

pub fn check_packages(packages: &[String]) -> Result<(), String> {
    if packages.len() > MAX_PACKAGES {
        return Err(format!("At most {MAX_PACKAGES} packages can be listed."));
    }
    match packages.iter().find(|name| !valid_package_name(name)) {
        Some(name) => Err(format!("'{name}' is not a valid package name.")),
        None => Ok(()),
    }
}

pub fn check_width(width: &str) -> Result<(), String> {
    crate::api::options::check_length(width, &WIDTH_UNITS)
}

fn package_lines<'a>(packages: impl Iterator<Item = &'a str>) -> String {
    let mut seen = Vec::new();
    for name in packages {
        if !seen.contains(&name) {
            seen.push(name);
        }
    }
    seen.iter().map(|name| format!("\\usepackage{{{name}}}\n")).collect()
}

fn assemble(class_options: &str, preamble: &str, before: &str, code: &str, after: &str) -> Wrapped {
    let header = format!("\\documentclass[{class_options}]{{standalone}}\n{preamble}\\begin{{document}}\n{before}");
    let lines_before_code = header.matches('\n').count();
    Wrapped { text: format!("{header}{code}{after}\n\\end{{document}}\n"), lines_before_code }
}

pub fn math_document(math: &str, display: bool, packages: &[String]) -> Wrapped {
    let names = MATH_PACKAGES.into_iter().chain(packages.iter().map(String::as_str));
    let style = if display { "\\displaystyle " } else { "" };
    assemble("border=2pt", &package_lines(names), &format!("${style}"), math.trim(), "$")
}

pub fn snippet_document(code: &str, preamble: Option<&str>, packages: &[String], width: Option<&str>) -> Wrapped {
    let class_options = match width {
        Some(width) => format!("border=4pt,varwidth={width}"),
        None => "border=4pt".to_string(),
    };
    let mut full_preamble = package_lines(packages.iter().map(String::as_str));
    if let Some(extra) = preamble.map(str::trim).filter(|extra| !extra.is_empty()) {
        full_preamble.push_str(extra);
        full_preamble.push('\n');
    }
    assemble(&class_options, &full_preamble, "", code.trim_end(), "")
}
