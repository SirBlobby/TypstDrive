export type DocumentLanguage = 'typst' | 'latex';

export function documentLanguage(value: string | null | undefined): DocumentLanguage {
    return value === 'latex' ? 'latex' : 'typst';
}

export function sourceExtension(language: DocumentLanguage) {
    return language === 'latex' ? 'tex' : 'typ';
}

export function languageLabel(language: DocumentLanguage) {
    return language === 'latex' ? 'LaTeX' : 'Typst';
}
