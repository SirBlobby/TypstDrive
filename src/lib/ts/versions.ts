import type { PersonSummary } from './people';

export type VersionKind = 'manual' | 'auto' | 'restore';

export interface DocumentVersion {
    id: string;
    document_id: string;
    author: PersonSummary | null;
    editors: PersonSummary[];
    content: string;
    created_at: string;
    updated_at: string | null;
    kind: VersionKind;
    label: string | null;
}

export const versionKindLabels: Record<VersionKind, string> = {
    manual: 'Saved',
    auto: 'Auto saved',
    restore: 'Restored'
};

async function errorText(res: Response, fallback: string) {
    return (await res.text()) || fallback;
}

export async function fetchVersions(docId: string): Promise<DocumentVersion[]> {
    const res = await fetch(`/api/docs/${docId}/versions`);
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load versions.'));
    return res.json();
}

export async function saveVersion(docId: string, label?: string): Promise<{ created: boolean; version: DocumentVersion | null }> {
    const res = await fetch(`/api/docs/${docId}/versions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ label: label ?? null })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to save the version.'));
    return res.json();
}

export async function renameVersion(docId: string, versionId: string, label: string) {
    const res = await fetch(`/api/docs/${docId}/versions/${versionId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ label })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to rename the version.'));
}

export async function restoreVersion(docId: string, versionId: string) {
    const res = await fetch(`/api/docs/${docId}/versions/${versionId}/restore`, { method: 'POST' });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to restore the version.'));
}

function parseTimestamp(value: string) {
    const normalized = value.includes('T') || value.endsWith('Z') ? value : value.replace(' ', 'T') + 'Z';
    return new Date(normalized);
}

function clockTime(value: string) {
    return parseTimestamp(value).toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' });
}

export function versionTime(version: DocumentVersion) {
    const startTime = clockTime(version.created_at);
    if (!version.updated_at) return startTime;
    const endTime = clockTime(version.updated_at);
    return endTime === startTime ? startTime : `${startTime} - ${endTime}`;
}

export function versionDay(version: DocumentVersion) {
    const date = parseTimestamp(version.updated_at ?? version.created_at);
    const today = new Date();
    const yesterday = new Date();
    yesterday.setDate(today.getDate() - 1);
    if (date.toDateString() === today.toDateString()) return 'Today';
    if (date.toDateString() === yesterday.toDateString()) return 'Yesterday';
    return date.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });
}

export function groupByDay(versions: DocumentVersion[]) {
    const groups: { day: string; versions: DocumentVersion[] }[] = [];
    for (const version of versions) {
        const day = versionDay(version);
        const last = groups[groups.length - 1];
        if (last && last.day === day) last.versions.push(version);
        else groups.push({ day, versions: [version] });
    }
    return groups;
}
