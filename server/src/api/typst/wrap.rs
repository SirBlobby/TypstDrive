pub const WIDTH_UNITS: [&str; 5] = ["pt", "mm", "cm", "in", "em"];

pub struct Wrapped {
    pub text: String,
    pub lines_before_code: usize,
}

pub struct Layout<'a> {
    pub preamble: Option<&'a str>,
    pub width: Option<&'a str>,
    pub transparent: bool,
    pub margin: &'a str,
}

fn assemble(layout: &Layout, code: &str) -> Wrapped {
    let width = layout.width.unwrap_or("auto");
    let fill = if layout.transparent { ", fill: none" } else { "" };
    let mut header = format!("#set page(width: {width}, height: auto, margin: {}{fill})\n", layout.margin);
    if let Some(preamble) = layout.preamble.map(str::trim).filter(|preamble| !preamble.is_empty()) {
        header.push_str(preamble);
        header.push('\n');
    }
    let lines_before_code = header.matches('\n').count();
    Wrapped { text: format!("{header}{code}\n"), lines_before_code }
}

pub fn math_document(math: &str, display: bool, layout: &Layout) -> Wrapped {
    let math = math.trim();
    let code = if display { format!("$ {math} $") } else { format!("${math}$") };
    assemble(layout, &code)
}

pub fn snippet_document(code: &str, layout: &Layout) -> Wrapped {
    assemble(layout, code.trim_end())
}
