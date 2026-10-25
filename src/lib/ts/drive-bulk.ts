import { itemName, type DriveCategory, type DriveEntry } from './drive-filters';
import { setStarred, starKey } from './stars';

const endpointBases: Record<DriveCategory, string> = {
    document: '/api/docs',
    folder: '/api/folders',
    file: '/api/files',
    project: '/api/projects'
};

export function itemEndpoint(category: DriveCategory, id: string) {
    return `${endpointBases[category]}/${encodeURIComponent(id)}`;
}

export function isMovable(entry: DriveEntry) {
    return entry.category !== 'folder';
}

export function isStarrable(entry: DriveEntry) {
    return entry.category === 'document' || entry.category === 'project';
}

export function describeEntries(entries: DriveEntry[]) {
    return entries.length === 1 ? `'${itemName(entries[0].item)}'` : `${entries.length} items`;
}

export async function deleteEntries(entries: DriveEntry[]) {
    let failed = 0;
    for (const entry of entries) {
        const res = await fetch(itemEndpoint(entry.category, entry.item.id), { method: 'DELETE' });
        if (!res.ok) failed++;
    }
    return failed;
}

export async function moveEntries(entries: DriveEntry[], folderId: string | null) {
    let failed = 0;
    for (const entry of entries.filter(isMovable)) {
        const res = await fetch(itemEndpoint(entry.category, entry.item.id), {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ folder_id: folderId ?? '' })
        });
        if (!res.ok) failed++;
    }
    return failed;
}

export function allStarred(entries: DriveEntry[], starred: Set<string>) {
    const starrable = entries.filter(isStarrable);
    return starrable.length > 0 && starrable.every((entry) => starred.has(starKey(entry.category as 'document' | 'project', entry.item.id)));
}

export async function starEntries(entries: DriveEntry[], starred: boolean) {
    for (const entry of entries.filter(isStarrable)) {
        await setStarred(entry.category as 'document' | 'project', entry.item.id, starred);
    }
}
