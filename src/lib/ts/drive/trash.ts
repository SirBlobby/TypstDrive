import { readable } from 'svelte/store';

export type TrashKind = 'document' | 'project' | 'file' | 'folder';

export interface TrashEntry {
    id: string;
    kind: TrashKind;
    item_id: string;
    name: string;
    original_folder_name: string | null;
    deleted_at: string;
    deleted_by_name: string | null;
    expires_at: string | null;
}

export interface TrashListing {
    retention_days: number;
    items: TrashEntry[];
}

export type TrashPolicy = { retention_days: number };

async function request<T>(url: string, method = 'GET', body?: unknown): Promise<T> {
    const res = await fetch(url, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Request failed');
    return res.status === 204 ? (undefined as T) : res.json();
}

export const fetchTrash = () => request<TrashListing>('/api/trash');
export const restoreTrashEntry = (id: string) => request<void>(`/api/trash/${encodeURIComponent(id)}/restore`, 'POST');
export const deleteTrashEntry = (id: string) => request<void>(`/api/trash/${encodeURIComponent(id)}`, 'DELETE');
export const emptyTrash = () => request<{ deleted: number }>('/api/trash', 'DELETE');

export const fetchTrashAdminPolicy = () => request<TrashPolicy>('/api/admin/trash-policy');
export const saveTrashAdminPolicy = (policy: TrashPolicy) => request<TrashPolicy>('/api/admin/trash-policy', 'PUT', policy);

export const trashRetentionStore = readable<number | null>(null, (set) => {
    request<TrashPolicy>('/api/trash/policy')
        .then((policy) => set(policy.retention_days))
        .catch(() => {});
});

export function deletionNotice(retentionDays: number | null) {
    if (retentionDays === 0) return 'This action cannot be undone.';
    if (retentionDays === null) return 'Deleted items can be restored from the trash.';
    return `Deleted items stay in the trash for ${retentionDays} ${retentionDays === 1 ? 'day' : 'days'}, where you can restore them.`;
}

function parseTimestamp(value: string) {
    const normalized = value.includes('T') || value.endsWith('Z') ? value : value.replace(' ', 'T') + 'Z';
    return new Date(normalized);
}

export function daysLeft(entry: TrashEntry) {
    if (!entry.expires_at) return null;
    const remaining = parseTimestamp(entry.expires_at).getTime() - Date.now();
    return Math.max(0, Math.ceil(remaining / (24 * 60 * 60 * 1000)));
}

export function formatDeletedAt(entry: TrashEntry) {
    const date = parseTimestamp(entry.deleted_at);
    return Number.isNaN(date.getTime()) ? entry.deleted_at : date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
}
