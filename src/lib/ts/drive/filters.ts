export type ItemType = 'all' | 'folder' | 'project' | 'document' | 'image' | 'pdf' | 'font' | 'other';
export type SortField = 'name' | 'modified' | 'size';
export type SortDirection = 'asc' | 'desc';

export type DriveCategory = 'folder' | 'project' | 'document' | 'file';
export type ViewMode = 'grid' | 'list';

export interface DriveEntry {
    category: DriveCategory;
    item: any;
}

export interface DriveFilters {
    sortField: SortField;
    sortDirection: SortDirection;
}

export const defaultFilters: DriveFilters = {
    sortField: 'name',
    sortDirection: 'asc'
};

export const sortOptions: { value: SortField; label: string }[] = [
    { value: 'name', label: 'Name' },
    { value: 'modified', label: 'Last modified' },
    { value: 'size', label: 'Size' }
];

function parseTimestamp(value?: string) {
    if (!value) return 0;
    const normalized = value.includes('T') || value.endsWith('Z') ? value : value.replace(' ', 'T') + 'Z';
    const time = Date.parse(normalized);
    return Number.isNaN(time) ? 0 : time;
}

export function itemName(item: any) {
    return (item.title ?? item.name ?? '') as string;
}

export function itemModified(item: any) {
    return parseTimestamp(item.updated_at ?? item.created_at);
}

export function itemSize(item: any) {
    return typeof item.size_bytes === 'number' ? item.size_bytes : 0;
}

export function fileType(file: any): ItemType {
    const mimeType = (file.mime_type ?? '').toLowerCase();
    const name = (file.name ?? '').toLowerCase();
    if (mimeType.startsWith('image/')) return 'image';
    if (mimeType === 'application/pdf' || name.endsWith('.pdf')) return 'pdf';
    if (mimeType.startsWith('font/') || /\.(ttf|otf|ttc|otc|woff2?)$/.test(name)) return 'font';
    return 'other';
}

export function sortItems<T>(items: T[], filters: DriveFilters, getItem: (entry: T) => any = (entry) => entry): T[] {
    const direction = filters.sortDirection === 'asc' ? 1 : -1;
    return [...items].sort((firstEntry, secondEntry) => {
        const first = getItem(firstEntry);
        const second = getItem(secondEntry);
        let difference = 0;
        if (filters.sortField === 'modified') difference = itemModified(first) - itemModified(second);
        else if (filters.sortField === 'size') difference = itemSize(first) - itemSize(second);
        if (difference === 0) {
            difference = itemName(first).localeCompare(itemName(second), undefined, { numeric: true, sensitivity: 'base' });
        }
        return difference * direction;
    });
}

export function formatSize(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    const units = ['KB', 'MB', 'GB'];
    let value = bytes / 1024;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
        value /= 1024;
        unitIndex++;
    }
    return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unitIndex]}`;
}
