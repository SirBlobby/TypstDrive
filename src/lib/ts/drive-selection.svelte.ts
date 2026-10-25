import type { DriveCategory, DriveEntry } from './drive-filters';

export interface ItemSelection {
    selected: boolean;
    selecting: boolean;
    click: (event: MouseEvent) => boolean;
    toggle: () => void;
}

export function selectionKey(category: DriveCategory, id: string) {
    return `${category}:${id}`;
}

export function entryKey(entry: DriveEntry) {
    return selectionKey(entry.category, entry.item.id);
}

export class DriveSelection {
    keys = $state<Set<string>>(new Set());
    private anchor: string | null = null;

    get count() {
        return this.keys.size;
    }

    get active() {
        return this.keys.size > 0;
    }

    has(key: string) {
        return this.keys.has(key);
    }

    clear() {
        if (this.keys.size > 0) this.keys = new Set();
        this.anchor = null;
    }

    toggle(key: string) {
        const next = new Set(this.keys);
        if (next.has(key)) next.delete(key);
        else next.add(key);
        this.keys = next;
        this.anchor = key;
    }

    selectRange(key: string, orderedKeys: string[]) {
        const start = this.anchor ? orderedKeys.indexOf(this.anchor) : -1;
        const end = orderedKeys.indexOf(key);
        if (start === -1 || end === -1) {
            this.toggle(key);
            return;
        }
        const [from, to] = start < end ? [start, end] : [end, start];
        this.keys = new Set([...this.keys, ...orderedKeys.slice(from, to + 1)]);
    }

    selectAll(orderedKeys: string[]) {
        this.keys = new Set(orderedKeys);
        this.anchor = orderedKeys[0] ?? null;
    }

    retain(availableKeys: Set<string>) {
        const kept = [...this.keys].filter((key) => availableKeys.has(key));
        if (kept.length !== this.keys.size) this.keys = new Set(kept);
    }

    handleClick(event: MouseEvent, key: string, orderedKeys: () => string[]) {
        if (event.shiftKey) {
            this.selectRange(key, orderedKeys());
            return true;
        }
        if (event.ctrlKey || event.metaKey || this.active) {
            this.toggle(key);
            return true;
        }
        return false;
    }

    itemSelection(key: string, orderedKeys: () => string[]): ItemSelection {
        return {
            selected: this.keys.has(key),
            selecting: this.keys.size > 0,
            click: (event) => this.handleClick(event, key, orderedKeys),
            toggle: () => this.toggle(key)
        };
    }
}
