import { writable } from 'svelte/store';

export type InsertDialog = 'image' | 'table' | 'document' | 'symbol' | 'emoji';

export const insertDialogStore = writable<InsertDialog | null>(null);

export const insertOptions: { dialog: InsertDialog; label: string; icon: string }[] = [
    { dialog: 'image', label: 'Image', icon: 'mdi:image-outline' },
    { dialog: 'table', label: 'Table', icon: 'mdi:table' },
    { dialog: 'document', label: 'Document', icon: 'mdi:file-document-outline' },
    { dialog: 'symbol', label: 'Symbol', icon: 'mdi:omega' },
    { dialog: 'emoji', label: 'Emoji', icon: 'mdi:emoticon-happy-outline' }
];
