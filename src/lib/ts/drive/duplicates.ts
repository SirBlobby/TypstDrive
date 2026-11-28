export type DuplicateKind = 'document' | 'project';

const endpoints: Record<DuplicateKind, string> = {
    document: '/api/docs',
    project: '/api/projects'
};

export async function duplicateItem(kind: DuplicateKind, id: string) {
    const res = await fetch(`${endpoints[kind]}/${encodeURIComponent(id)}/duplicate`, { method: 'POST' });
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Could not duplicate this item');
    const created: { id: string } = await res.json();
    return created.id;
}

export function duplicatePath(kind: DuplicateKind, id: string) {
    return kind === 'document' ? `/doc/${id}` : `/project/${id}`;
}
