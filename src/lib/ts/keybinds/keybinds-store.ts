import { get, writable } from 'svelte/store';
import { keybindActions, type KeybindActionId } from './keybind-actions';
import { normalizeKey, shortcutText } from './key-format';

export type Keybinds = Record<KeybindActionId, string>;

export const defaultKeybinds = Object.fromEntries(
    keybindActions.map((action) => [action.id, action.defaultKey])
) as Keybinds;

const STORAGE_KEY = 'editor-keybinds';

function loadSaved(): Keybinds {
    if (typeof window === 'undefined') return defaultKeybinds;
    try {
        const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
        const keybinds = { ...defaultKeybinds };
        for (const action of keybindActions) {
            if (typeof saved[action.id] === 'string') keybinds[action.id] = saved[action.id];
        }
        return keybinds;
    } catch {
        return defaultKeybinds;
    }
}

export const keybindsStore = writable<Keybinds>(loadSaved());

if (typeof window !== 'undefined') {
    keybindsStore.subscribe((keybinds) => {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(keybinds));
        } catch {}
    });
}

export function setKeybind(id: KeybindActionId, key: string) {
    keybindsStore.update((keybinds) => ({ ...keybinds, [id]: key }));
}

export function resetKeybinds() {
    keybindsStore.set(defaultKeybinds);
}

export function currentKeybind(id: KeybindActionId) {
    return get(keybindsStore)[id];
}

export function withShortcut(label: string, id: KeybindActionId) {
    const shortcut = shortcutText(currentKeybind(id));
    return shortcut ? `${label} (${shortcut})` : label;
}

export function conflictingActions(keybinds: Keybinds) {
    const actionsByKey = new Map<string, KeybindActionId[]>();
    for (const action of keybindActions) {
        const key = normalizeKey(keybinds[action.id]);
        if (!key) continue;
        actionsByKey.set(key, [...(actionsByKey.get(key) ?? []), action.id]);
    }
    return new Set([...actionsByKey.values()].filter((ids) => ids.length > 1).flat());
}
