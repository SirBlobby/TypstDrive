export type ItemKind = 'document' | 'project' | 'folder' | 'file';

export type InfoPerson = {
    id: string;
    name: string;
    email?: string | null;
    is_team: boolean;
};

export type InfoAccessEntry = {
    name: string;
    email?: string | null;
    role: string;
    source: 'owner' | 'team' | 'shared' | 'folder';
    via?: string | null;
};

export type ItemInfo = {
    kind: ItemKind;
    name: string;
    owner: InfoPerson;
    your_role: string;
    size_bytes: number;
    location: string[] | null;
    created_at: string;
    updated_at?: string | null;
    last_edited_by?: InfoPerson | null;
    mime_type?: string | null;
    link_access?: string | null;
    access: InfoAccessEntry[];
};

export async function fetchItemInfo(kind: ItemKind, id: string): Promise<ItemInfo> {
    const res = await fetch(`/api/item-info/${kind}/${id}`);
    if (!res.ok) throw new Error((await res.text()) || 'Could not load details');
    return res.json();
}

export async function replaceFileData(fileId: string, file: File) {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`/api/files/${fileId}/data`, { method: 'PUT', body: formData });
    if (!res.ok) throw new Error((await res.text()) || 'Could not replace the file');
    return res.json();
}

export function fileDataUrl(file: { id: string; updated_at?: string | null }) {
    const version = file.updated_at ? `?v=${encodeURIComponent(file.updated_at)}` : '';
    return `/api/files/${file.id}/data${version}`;
}

export function formatTimestamp(value: string) {
    const normalized = value.includes('T') || value.endsWith('Z') ? value : value.replace(' ', 'T') + 'Z';
    const date = new Date(normalized);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}
