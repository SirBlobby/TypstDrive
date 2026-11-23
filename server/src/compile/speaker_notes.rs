//! Speaker notes per page, read from the metadata that Typst slide packages emit.
//!
//! - Touying and Polylux: `metadata((t: "Note", v: text)) <pdfpc>`, one per note.
//! - Mosaic: `metadata((kind: "speaker-notes", notes: (..))) <mosaic-speaker-notes>`, one per page.
//!
//! Notes are typeset with Typst, so lists, emphasis and math look as written. Each note's
//! markup goes into a small generated document, one page per note: the original source for
//! content notes, the string itself for text notes.

use std::fmt::Write;
use std::ops::Range;

use serde::Serialize;
use typst::diag::Warned;
use typst::foundations::{Content, Label, SequenceElem, Selector, Value};
use typst::introspection::Introspector;
use typst::utils::PicoStr;
use typst::{World, WorldExt};
use typst_layout::PagedDocument;

use crate::compile::compiler::svg_options;
use crate::compile::world::MemoryWorld;

const PDFPC_LABEL: &str = "pdfpc";
const MOSAIC_LABEL: &str = "mosaic-speaker-notes";
const NOTE_LABEL: &str = "typstdrive-speaker-note";
const NOTES_FILE: &str = "__typstdrive_speaker_notes__.typ";

/// Sized for the notes pane: the frontend scales the page with the notes font size.
const NOTES_PREAMBLE: &str = "#set page(width: 300pt, height: auto, margin: (x: 0pt, y: 4pt), fill: none)\n\
#set text(size: 11pt, fill: white)\n";

#[derive(Serialize, Clone)]
pub struct Note {
    /// Plain text, shown when the note can't be typeset.
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg: Option<String>,
}

struct NoteSource {
    markup: String,
    text: String,
}

/// One entry per page, or `None` when the document has no notes at all.
pub fn page_notes(world: &MemoryWorld, document: &PagedDocument) -> Option<Vec<Option<Note>>> {
    let mut pages: Vec<Vec<NoteSource>> = (0..document.pages().len()).map(|_| Vec::new()).collect();

    for (page, value) in labelled_values(document, PDFPC_LABEL) {
        if let Some(note) = pdfpc_note(world, &value) {
            push_note(&mut pages, page, note);
        }
    }
    for (page, value) in labelled_values(document, MOSAIC_LABEL) {
        for note in mosaic_notes(world, &value) {
            push_note(&mut pages, page, note);
        }
    }

    if pages.iter().all(Vec::is_empty) {
        return None;
    }
    let pages: Vec<Option<NoteSource>> = pages.into_iter().map(merge_notes).collect();
    let svgs = typeset(world, &pages);
    Some(
        pages
            .into_iter()
            .zip(svgs)
            .map(|(note, svg)| note.map(|note| Note { text: note.text, svg }))
            .collect(),
    )
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

fn pdfpc_note(world: &MemoryWorld, value: &Value) -> Option<NoteSource> {
    let Value::Dict(dict) = value else { return None };
    match dict.get("t").ok()? {
        Value::Str(kind) if kind.as_str() == "Note" => note_source(world, dict.get("v").ok()?),
        _ => None,
    }
}

fn mosaic_notes(world: &MemoryWorld, value: &Value) -> Vec<NoteSource> {
    let Value::Dict(dict) = value else { return Vec::new() };
    match dict.get("notes") {
        Ok(Value::Array(notes)) => notes.iter().filter_map(|note| note_source(world, note)).collect(),
        _ => Vec::new(),
    }
}

fn note_source(world: &MemoryWorld, value: &Value) -> Option<NoteSource> {
    let (markup, text) = match value {
        Value::Str(text) => {
            let text = text.as_str().trim().to_string();
            (text.clone(), text)
        }
        Value::Content(content) => {
            let text = content.plain_text().trim().to_string();
            let markup = source_markup(world, content).unwrap_or_else(|| string_literal(&text));
            (markup, text)
        }
        _ => return None,
    };
    (!text.is_empty()).then_some(NoteSource { markup, text })
}

/// The note's markup exactly as written, e.g. the body of `#note[...]`.
///
/// Gives up when the pieces aren't one contiguous run of source, such as a note that
/// inserts a variable defined elsewhere, since the generated document can't resolve it.
fn source_markup(world: &MemoryWorld, content: &Content) -> Option<String> {
    let parts = match content.to_packed::<SequenceElem>() {
        Some(sequence) => sequence.children.iter().collect(),
        None => vec![content],
    };
    let id = parts.first()?.span().id()?;
    let source = world.source(id).ok()?;
    let text = source.text();

    let mut range: Option<Range<usize>> = None;
    for part in parts {
        let span = part.span();
        if span.id() != Some(id) {
            return None;
        }
        let next = world.range(span)?;
        range = Some(match range {
            None => next,
            // Only whitespace may sit between the parts the note itself wrote.
            Some(current) if text.get(current.end..next.start)?.trim().is_empty() => current.start..next.end,
            Some(_) => return None,
        });
    }
    let range = range?;

    // Keep the first line's indentation so nested lists stay nested after dedenting.
    let line_start = text[..range.start].rfind('\n').map_or(0, |index| index + 1);
    let start = if text[line_start..range.start].trim().is_empty() { line_start } else { range.start };
    Some(dedent(text.get(start..range.end)?))
}

fn dedent(text: &str) -> String {
    let indent = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    text.lines()
        .map(|line| line.get(indent..).unwrap_or(line.trim_start()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Typst markup that shows `text` verbatim.
fn string_literal(text: &str) -> String {
    let mut literal = String::from("#\"");
    for c in text.chars() {
        match c {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => {}
            '\t' => literal.push_str("\\t"),
            c => literal.push(c),
        }
    }
    literal.push('"');
    literal
}

fn push_note(pages: &mut [Vec<NoteSource>], page: usize, note: NoteSource) {
    if let Some(notes) = pages.get_mut(page) {
        notes.push(note);
    }
}

fn merge_notes(notes: Vec<NoteSource>) -> Option<NoteSource> {
    if notes.is_empty() {
        return None;
    }
    let (markup, text): (Vec<_>, Vec<_>) = notes.into_iter().map(|note| (note.markup, note.text)).unzip();
    Some(NoteSource { markup: markup.join("\n\n"), text: text.join("\n\n") })
}

/// An SVG per note. When the batch fails, each note is retried alone so one broken
/// note doesn't take the others down with it.
fn typeset(world: &MemoryWorld, notes: &[Option<NoteSource>]) -> Vec<Option<String>> {
    let pending: Vec<(usize, &str)> = notes
        .iter()
        .enumerate()
        .filter_map(|(index, note)| Some((index, note.as_ref()?.markup.as_str())))
        .collect();
    let mut svgs = vec![None; notes.len()];
    let typeset = typeset_batch(world, &pending).unwrap_or_else(|| {
        pending
            .iter()
            .filter_map(|&note| typeset_batch(world, &[note]))
            .flatten()
            .collect()
    });
    for (index, svg) in typeset {
        if let Some(slot) = svgs.get_mut(index) {
            *slot = Some(svg);
        }
    }
    svgs
}

fn typeset_batch(world: &MemoryWorld, notes: &[(usize, &str)]) -> Option<Vec<(usize, String)>> {
    let mut source = String::from(NOTES_PREAMBLE);
    for (index, markup) in notes {
        // Each note sits in its own block, so its set rules and errors stay local.
        let _ = write!(source, "#pagebreak(weak: true)\n#metadata({index}) <{NOTE_LABEL}>\n#[\n{markup}\n]\n");
    }
    let world = world.with_main(NOTES_FILE, source);
    let Warned { output, warnings: _ } = typst::compile::<PagedDocument>(&world);
    let document = output.ok()?;
    let options = svg_options(&document);
    Some(
        labelled_values(&document, NOTE_LABEL)
            .into_iter()
            .filter_map(|(page, value)| {
                let Value::Int(index) = value else { return None };
                Some((usize::try_from(index).ok()?, typst_svg::svg(document.pages().get(page)?, &options)))
            })
            .collect(),
    )
}
