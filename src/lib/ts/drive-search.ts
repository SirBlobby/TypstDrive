import { fileType, type DriveCategory } from './drive-filters';
import { starKey } from './stars';

export const MINIMUM_CONTENT_QUERY_LENGTH = 2;

export type SearchFilterKey = 'doc' | 'type' | 'is' | 'before' | 'after';

export interface SearchToken {
    key: SearchFilterKey;
    value: string;
}

export interface DriveSearch {
    text: string;
    tokens: SearchToken[];
}

export interface SearchFilterValue {
    value: string;
    description: string;
}

export interface SearchFilterDefinition {
    key: SearchFilterKey;
    description: string;
    icon: string;
    hint: string;
    values?: SearchFilterValue[];
    isDate?: boolean;
    allowsSpaces?: boolean;
}

export interface ContentSearchResult {
    id: string;
    title: string;
    folder_id: string | null;
    updated_at: string;
    language: string | null;
    shared: boolean;
    match_count: number;
    snippet: string;
}

export const searchFilters: SearchFilterDefinition[] = [
    { key: 'doc', description: 'Text inside documents', icon: 'mdi:text-search', hint: 'Type a phrase, then press Enter', allowsSpaces: true },
    {
        key: 'type',
        description: 'Kind of item',
        icon: 'mdi:shape-outline',
        hint: 'Pick a type',
        values: [
            { value: 'document', description: 'Any document' },
            { value: 'typst', description: 'Typst documents' },
            { value: 'latex', description: 'LaTeX documents' },
            { value: 'project', description: 'Projects' },
            { value: 'folder', description: 'Folders' },
            { value: 'file', description: 'Any uploaded file' },
            { value: 'image', description: 'Images' },
            { value: 'pdf', description: 'PDF files' },
            { value: 'font', description: 'Font files' }
        ]
    },
    {
        key: 'is',
        description: 'Item state',
        icon: 'mdi:tag-outline',
        hint: 'Pick a state',
        values: [
            { value: 'starred', description: 'Starred documents and projects' },
            { value: 'shared', description: 'Shared with you' }
        ]
    },
    { key: 'before', description: 'Modified before a date', icon: 'mdi:calendar-arrow-left', hint: 'YYYY-MM-DD', isDate: true },
    { key: 'after', description: 'Modified after a date', icon: 'mdi:calendar-arrow-right', hint: 'YYYY-MM-DD', isDate: true }
];

const DOCUMENT_TYPES = ['document', 'typst', 'latex'];
const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;
const TOKEN_PATTERN = /^([a-z]+):(.*)$/i;

export function emptySearch(): DriveSearch {
    return { text: '', tokens: [] };
}

export function findFilter(key: string) {
    return searchFilters.find((filter) => filter.key === key.toLowerCase());
}

export function parseTokenDraft(text: string): { filter: SearchFilterDefinition; value: string } | null {
    const match = text.trimStart().match(TOKEN_PATTERN);
    if (!match) return null;
    const filter = findFilter(match[1]);
    if (!filter) return null;
    return { filter, value: filter.allowsSpaces ? match[2].trim() : match[2].trim().toLowerCase() };
}

export function isValidTokenValue(filter: SearchFilterDefinition, value: string) {
    if (filter.values) return filter.values.some((option) => option.value === value);
    if (filter.isDate) return DATE_PATTERN.test(value) && !Number.isNaN(Date.parse(value));
    return value.length >= MINIMUM_CONTENT_QUERY_LENGTH;
}

export function withToken(search: DriveSearch, token: SearchToken): DriveSearch {
    const otherTokens = search.tokens.filter((existing) => existing.key !== token.key);
    return { text: '', tokens: [...otherTokens, token] };
}

export function withoutToken(search: DriveSearch, index: number): DriveSearch {
    return { ...search, tokens: search.tokens.filter((_, tokenIndex) => tokenIndex !== index) };
}

export function hasSearch(search: DriveSearch) {
    return search.text.trim().length > 0 || search.tokens.length > 0;
}

export function tokenValue(search: DriveSearch, key: SearchFilterKey) {
    return search.tokens.find((token) => token.key === key)?.value ?? null;
}

export function nameText(search: DriveSearch) {
    return parseTokenDraft(search.text) ? '' : search.text.trim().toLowerCase();
}

function itemDay(item: any) {
    return String(item.updated_at ?? item.created_at ?? '').slice(0, 10);
}

function matchesType(type: string, item: any, category: DriveCategory) {
    if (type === 'folder' || type === 'project' || type === 'document') return category === type;
    if (type === 'typst') return category === 'document' && item.language !== 'latex';
    if (type === 'latex') return category === 'document' && item.language === 'latex';
    if (type === 'file') return category === 'file';
    return category === 'file' && fileType(item) === type;
}

function matchesState(state: string, item: any, category: DriveCategory, starredKeys: Set<string>) {
    if (state === 'starred') {
        if (category !== 'document' && category !== 'project') return false;
        return starredKeys.has(starKey(category, item.id));
    }
    return Boolean(item.effective_role) && item.effective_role !== 'owner';
}

export function matchesItem(search: DriveSearch, item: any, category: DriveCategory, starredKeys: Set<string>) {
    const text = nameText(search);
    if (text && !(item.title ?? item.name ?? '').toLowerCase().includes(text)) return false;

    for (const token of search.tokens) {
        if (token.key === 'type' && !matchesType(token.value, item, category)) return false;
        if (token.key === 'is' && !matchesState(token.value, item, category, starredKeys)) return false;
        if (token.key === 'before' && !(itemDay(item) < token.value)) return false;
        if (token.key === 'after' && !(itemDay(item) > token.value)) return false;
    }
    return true;
}

export function contentTypeAllowed(search: DriveSearch) {
    const type = tokenValue(search, 'type');
    return type === null || DOCUMENT_TYPES.includes(type);
}

export function contentSearchParams(search: DriveSearch, text: string) {
    const params = new URLSearchParams({ q: text });
    const type = tokenValue(search, 'type');
    if (type === 'typst' || type === 'latex') params.set('language', type);
    if (tokenValue(search, 'is') === 'shared') params.set('shared', 'true');
    const before = tokenValue(search, 'before');
    const after = tokenValue(search, 'after');
    if (before) params.set('before', before);
    if (after) params.set('after', after);
    return params;
}

export async function searchDocumentText(params: URLSearchParams, signal: AbortSignal): Promise<ContentSearchResult[]> {
    const response = await fetch(`/api/docs/search?${params}`, { signal });
    if (!response.ok) throw new Error(await response.text());
    return response.json();
}

export function highlightParts(snippet: string, text: string): { text: string; matched: boolean }[] {
    if (!text) return [{ text: snippet, matched: false }];
    const lowerSnippet = snippet.toLowerCase();
    const lowerText = text.toLowerCase();
    const parts: { text: string; matched: boolean }[] = [];
    let position = 0;
    let matchIndex = lowerSnippet.indexOf(lowerText);
    while (matchIndex !== -1) {
        if (matchIndex > position) parts.push({ text: snippet.slice(position, matchIndex), matched: false });
        parts.push({ text: snippet.slice(matchIndex, matchIndex + text.length), matched: true });
        position = matchIndex + text.length;
        matchIndex = lowerSnippet.indexOf(lowerText, position);
    }
    if (position < snippet.length) parts.push({ text: snippet.slice(position), matched: false });
    return parts;
}
