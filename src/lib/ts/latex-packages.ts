import { get } from 'svelte/store';
import { EditorState, type Extension, type Text, type Transaction, type TransactionSpec } from '@codemirror/state';
import { ySyncAnnotation } from 'y-codemirror.next';
import { suggestingModeStore } from './suggestions';

interface PackageRule {
    name: string;
    needed: RegExp;
}

const PACKAGE_RULES: PackageRule[] = [
    { name: 'graphicx', needed: /\\includegraphics\b/ },
    { name: 'hyperref', needed: /\\(href|url)\b/ },
    { name: 'fontspec', needed: /\\(setmainfont|setsansfont|setmonofont|newfontfamily|fontspec)\b/ },
    { name: 'amsmath', needed: /\\(eqref|text)\b|\\begin\{(align|gather|multline)\*?\}/ }
];

const DOCUMENT_CLASS = /^[ \t]*\\documentclass\b/m;
const BEGIN_DOCUMENT = '\\begin{document}';
const PACKAGE_LINE = /^[ \t]*\\(?:usepackage|RequirePackage)\s*(?:\[[^\]]*\])?\s*\{([^}]*)\}/gm;
const SKIPPED_EVENTS = ['input.type', 'delete', 'undo', 'redo', 'move'];

function preambleOf(doc: Text) {
    const text = doc.toString();
    const end = text.indexOf(BEGIN_DOCUMENT);
    if (end === -1) return null;
    const preamble = text.slice(0, end);
    return DOCUMENT_CLASS.test(preamble) ? preamble : null;
}

function loadedPackages(preamble: string) {
    const loaded = new Set<string>();
    for (const match of preamble.matchAll(PACKAGE_LINE)) {
        for (const name of match[1].split(',')) loaded.add(name.trim());
    }
    return loaded;
}

function insertionPoint(doc: Text, preamble: string) {
    const lines = [...preamble.matchAll(PACKAGE_LINE)];
    const anchor = lines.length > 0 ? lines[lines.length - 1].index : preamble.search(DOCUMENT_CLASS);
    return doc.lineAt(anchor).to;
}

function insertedText(transaction: Transaction) {
    let text = '';
    transaction.changes.iterChanges((_fromA, _toA, _fromB, _toB, inserted) => {
        text += inserted.toString();
    });
    return text;
}

function shouldCheck(transaction: Transaction) {
    if (!transaction.docChanged || transaction.annotation(ySyncAnnotation) !== undefined) return false;
    if (SKIPPED_EVENTS.some((event) => transaction.isUserEvent(event))) return false;
    return !get(suggestingModeStore);
}

function missingPackagesSpec(transaction: Transaction): TransactionSpec | null {
    const inserted = insertedText(transaction);
    const needed = PACKAGE_RULES.filter((rule) => rule.needed.test(inserted));
    if (needed.length === 0) return null;

    const doc = transaction.newDoc;
    const preamble = preambleOf(doc);
    if (!preamble) return null;
    const loaded = loadedPackages(preamble);
    const missing = needed.filter((rule) => !loaded.has(rule.name));
    if (missing.length === 0) return null;

    const lines = missing.map((rule) => `\n\\usepackage{${rule.name}}`).join('');
    return { changes: { from: insertionPoint(doc, preamble), insert: lines }, sequential: true };
}

export function latexPackageLoader(): Extension {
    return EditorState.transactionFilter.of((transaction) => {
        if (!shouldCheck(transaction)) return transaction;
        const spec = missingPackagesSpec(transaction);
        return spec ? [transaction, spec] : transaction;
    });
}
