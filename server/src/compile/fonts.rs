use std::sync::OnceLock;

use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst_kit::fonts::FontStore;

pub struct SharedFonts {
    store: FontStore,
    count: usize,
}

impl SharedFonts {
    pub fn book(&self) -> FontBook {
        (**self.store.book()).clone()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn font(&self, index: usize) -> Option<Font> {
        self.store.font(index)
    }
}

pub fn shared_fonts() -> &'static SharedFonts {
    static FONTS: OnceLock<SharedFonts> = OnceLock::new();
    FONTS.get_or_init(|| {
        let embedded_fonts: Vec<_> = typst_kit::fonts::embedded().collect();
        let system_fonts: Vec<_> = typst_kit::fonts::system().collect();
        let count = embedded_fonts.len() + system_fonts.len();

        let mut store = FontStore::new();
        store.extend(embedded_fonts);
        store.extend(system_fonts);

        tracing::info!("Loaded {count} shared fonts (embedded and system)");
        SharedFonts { store, count }
    })
}

pub fn shared_book() -> &'static LazyHash<FontBook> {
    static BOOK: OnceLock<LazyHash<FontBook>> = OnceLock::new();
    BOOK.get_or_init(|| LazyHash::new(shared_fonts().book()))
}

pub fn is_font_data(data: &[u8]) -> bool {
    let has_font_signature = matches!(data.get(..4), Some(b"\x00\x01\x00\x00" | b"OTTO" | b"true" | b"ttcf"));
    has_font_signature && Font::iter(typst::foundations::Bytes::new(data.to_vec())).next().is_some()
}
