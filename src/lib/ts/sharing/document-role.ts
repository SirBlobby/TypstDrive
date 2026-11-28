export type DocumentRole = 'owner' | 'editor' | 'suggester' | 'viewer';

export const documentRoleLabels: Record<DocumentRole, string> = {
    owner: 'Owner',
    editor: 'Editor',
    suggester: 'Suggester',
    viewer: 'Viewer'
};
