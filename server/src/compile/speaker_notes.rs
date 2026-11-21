//! Speaker notes per page, read from the metadata that Typst slide packages emit.
//!
//! - Touying and Polylux: `metadata((t: "Note", v: text)) <pdfpc>`, one per note.
//! - Mosaic: `metadata((kind: "speaker-notes", notes: (..))) <mosaic-speaker-notes>`, one per page.

use typst::foundations::{Label, Selector, Value};
use typst::introspection::Introspector;
use typst::utils::PicoStr;
use typst_layout::PagedDocument;

const PDFPC_LABEL: &str = "pdfpc";
const MOSAIC_LABEL: &str = "mosaic-speaker-notes";

/// One entry per page, or `None` when the document has no notes at all.
pub fn page_notes(document: &PagedDocument) -> Option<Vec<Option<String>>> {
    let mut pages: Vec<Vec<String>> = vec![Vec::new(); document.pages().len()];

    for (page, value) in labelled_values(document, PDFPC_LABEL) {
        if let Some(note) = pdfpc_note(&value) {
            push_note(&mut pages, page, note);
        }
    }
    for (page, value) in labelled_values(document, MOSAIC_LABEL) {
        for note in mosaic_notes(&value) {
            push_note(&mut pages, page, note);
        }
    }

    if pages.iter().all(Vec::is_empty) {
        return None;
    }
    Some(pages.into_iter().map(|notes| (!notes.is_empty()).then(|| notes.join("\n\n"))).collect())
}

/// Metadata values carrying `label`, paired with their zero-based page index.
fn labelled_values(document: &PagedDocument, label: &str) -> Vec<(usize, Value)> {
    let Some(label) = Label::new(PicoStr::intern(label)) else {
        return Vec::new();
    };
    let introspector = document.introspector();
    introspector
        .query(&Selector::Label(label))
        .iter()
        .filter_map(|content| {
            let page = introspector.page(content.location()?)?.get() - 1;
            let value = content.get_by_name("value").ok()?;
            Some((page, value))
        })
        .collect()
}

fn pdfpc_note(value: &Value) -> Option<String> {
    let Value::Dict(dict) = value else { return None };
    match dict.get("t").ok()? {
        Value::Str(kind) if kind.as_str() == "Note" => dict.get("v").ok().and_then(value_text),
        _ => None,
    }
}

fn mosaic_notes(value: &Value) -> Vec<String> {
    let Value::Dict(dict) = value else { return Vec::new() };
    match dict.get("notes") {
        Ok(Value::Array(notes)) => notes.iter().filter_map(value_text).collect(),
        _ => Vec::new(),
    }
}

fn value_text(value: &Value) -> Option<String> {
    let text = match value {
        Value::Str(text) => text.to_string(),
        Value::Content(content) => content.plain_text().to_string(),
        _ => return None,
    };
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn push_note(pages: &mut [Vec<String>], page: usize, note: String) {
    if let Some(notes) = pages.get_mut(page) {
        notes.push(note);
    }
}
