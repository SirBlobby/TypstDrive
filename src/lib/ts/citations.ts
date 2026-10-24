import { writable } from 'svelte/store';

export type CitationTargetKind = 'document' | 'project';
export type BibliographyFormat = 'bibtex' | 'hayagriva';

export type CitationPolicy = {
    lookup_enabled: boolean;
    zotero_enabled: boolean;
};

export type LookupResult = {
    key: string;
    title: string;
    format: BibliographyFormat;
    entry: string;
};

export type AddedEntry = {
    key: string;
    title: string;
    file_name: string;
    entry: string;
    created_file: boolean;
};

export type ZoteroStatus = {
    enabled: boolean;
    connected: boolean;
    username: string | null;
};

export type ZoteroLibrary = { id: string; name: string };
export type ZoteroCollection = { key: string; name: string; parent_key: string | null };

export type CitationSync = {
    id: string;
    library_name: string;
    collection_name: string | null;
    file_name: string;
    item_count: number;
    last_synced_at: string | null;
    created_by_name: string | null;
    is_mine: boolean;
};

export type NewSync = {
    library_id: string;
    library_name: string;
    collection_key: string | null;
    collection_name: string | null;
    file_name: string;
};

export const bibliographyChangedStore = writable(0);

export function notifyBibliographyChanged() {
    bibliographyChangedStore.update((count) => count + 1);
}

export function formatForFile(fileName: string): BibliographyFormat {
    return fileName.toLowerCase().endsWith('.bib') ? 'bibtex' : 'hayagriva';
}

export function citationMarkup(key: string, latex: boolean): string {
    return latex ? `\\cite{${key}}` : `@${key}`;
}

async function request<T>(url: string, method = 'GET', body?: unknown): Promise<T> {
    const res = await fetch(url, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Request failed');
    return res.status === 204 ? (undefined as T) : res.json();
}

const targetBase = (kind: CitationTargetKind, id: string) => `/api/citations/${kind}/${id}`;

export const fetchCitationPolicy = () => request<CitationPolicy>('/api/citations/policy');
export const lookupCitation = (identifier: string, format: BibliographyFormat) =>
    request<LookupResult>('/api/citations/lookup', 'POST', { identifier, format });
export const fetchBibliographyFiles = (kind: CitationTargetKind, id: string) => request<string[]>(`${targetBase(kind, id)}/files`);
export const addCitation = (kind: CitationTargetKind, id: string, identifier: string, fileName: string) =>
    request<AddedEntry>(`${targetBase(kind, id)}/entries`, 'POST', { identifier, file_name: fileName });

export const fetchZoteroStatus = () => request<ZoteroStatus>('/api/citations/zotero');
export const connectZotero = (apiKey: string) => request<ZoteroStatus>('/api/citations/zotero', 'PUT', { api_key: apiKey });
export const disconnectZotero = () => request<void>('/api/citations/zotero', 'DELETE');
export const fetchZoteroLibraries = () => request<ZoteroLibrary[]>('/api/citations/zotero/libraries');
export const fetchZoteroCollections = (library: string) =>
    request<ZoteroCollection[]>(`/api/citations/zotero/collections?library=${encodeURIComponent(library)}`);

export const fetchSyncs = (kind: CitationTargetKind, id: string) => request<CitationSync[]>(`${targetBase(kind, id)}/syncs`);
export const createSync = (kind: CitationTargetKind, id: string, sync: NewSync) => request<CitationSync>(`${targetBase(kind, id)}/syncs`, 'POST', sync);
export const runSync = (syncId: string) => request<CitationSync>(`/api/citations/syncs/${syncId}/run`, 'POST');
export const deleteSync = (syncId: string) => request<void>(`/api/citations/syncs/${syncId}`, 'DELETE');

export const fetchCitationAdminPolicy = () => request<CitationPolicy>('/api/admin/citation-policy');
export const saveCitationAdminPolicy = (policy: CitationPolicy) => request<CitationPolicy>('/api/admin/citation-policy', 'PUT', policy);

export function collectionLabels(collections: ZoteroCollection[]): { key: string; label: string }[] {
    const byKey = new Map(collections.map((collection) => [collection.key, collection]));
    const pathOf = (collection: ZoteroCollection, depth = 0): string => {
        const parent = collection.parent_key ? byKey.get(collection.parent_key) : undefined;
        return parent && depth < 10 ? `${pathOf(parent, depth + 1)} / ${collection.name}` : collection.name;
    };
    return collections
        .map((collection) => ({ key: collection.key, label: pathOf(collection) }))
        .sort((first, second) => first.label.localeCompare(second.label));
}
