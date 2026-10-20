use std::sync::OnceLock;

use typst::text::{Font, FontBook};
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
