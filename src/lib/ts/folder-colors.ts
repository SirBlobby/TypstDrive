export interface FolderColorOption {
    label: string;
    value: string | null;
}

export const folderColorOptions: FolderColorOption[] = [
    { label: 'Default', value: null },
    { label: 'Red', value: '#ef4444' },
    { label: 'Orange', value: '#f97316' },
    { label: 'Green', value: '#22c55e' },
    { label: 'Teal', value: '#14b8a6' },
    { label: 'Blue', value: '#3b82f6' },
    { label: 'Indigo', value: '#6366f1' },
    { label: 'Purple', value: '#a855f7' },
    { label: 'Pink', value: '#ec4899' },
    { label: 'Gray', value: '#6b7280' }
];

export const defaultFolderColor = '#eab308';

export function folderIconStyle(color: string | null | undefined) {
    return `color: ${color ?? defaultFolderColor};`;
}

export function folderTileStyle(color: string | null | undefined) {
    return `background-color: ${color ?? defaultFolderColor}1a;`;
}

export async function saveFolderColor(folderId: string, color: string | null) {
    const response = await fetch(`/api/folders/${folderId}/color`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ color })
    });
    if (!response.ok) throw new Error((await response.text()) || 'Could not change the folder color');
    const result: { color: string | null } = await response.json();
    return result.color;
}
