import { writable } from 'svelte/store';

export type StarKind = 'document' | 'project';

export type StarredItems = {
    documents: any[];
    projects: any[];
};

export const starredStore = writable<Set<string>>(new Set());

export function starKey(kind: StarKind, id: string) {
    return `${kind}:${id}`;
}

export async function fetchStarred(): Promise<StarredItems> {
    const res = await fetch('/api/stars');
    if (!res.ok) return { documents: [], projects: [] };

    const items: StarredItems = await res.json();
    starredStore.set(
        new Set([
            ...items.documents.map((document) => starKey('document', document.id)),
            ...items.projects.map((project) => starKey('project', project.id))
        ])
    );
    return items;
}

export async function setStarred(kind: StarKind, id: string, starred: boolean) {
    const res = await fetch(`/api/stars/${kind}/${id}`, { method: starred ? 'PUT' : 'DELETE' });
    if (!res.ok) return false;

    starredStore.update((keys) => {
        const next = new Set(keys);
        if (starred) next.add(starKey(kind, id));
        else next.delete(starKey(kind, id));
        return next;
    });
    return true;
}
