const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);

const modifierKeys = ['Control', 'Shift', 'Alt', 'Meta', 'AltGraph', 'CapsLock'];
const modifierOrder = ['Mod', 'Ctrl', 'Alt', 'Shift'];

const modifierLabels: Record<string, string> = {
    Mod: isMac ? 'Cmd' : 'Ctrl',
    Ctrl: 'Ctrl',
    Alt: isMac ? 'Option' : 'Alt',
    Shift: 'Shift'
};

function splitKey(key: string) {
    const parts = key.split(/-(?!$)/);
    const keyName = parts.pop() ?? '';
    const modifiers = parts.map((part) => (part === 'Ctrl' && !isMac ? 'Mod' : part));
    return { modifiers: [...new Set(modifiers)], keyName };
}

export function normalizeKey(key: string) {
    if (!key) return '';
    const { modifiers, keyName } = splitKey(key);
    const sorted = modifierOrder.filter((modifier) => modifiers.includes(modifier));
    return [...sorted, keyName.length === 1 ? keyName.toLowerCase() : keyName].join('-');
}

function keyNameFromEvent(event: KeyboardEvent) {
    if (event.key === ' ') return 'Space';
    const typedSpecialCharacter = event.altKey && isMac && !/^[\x21-\x7e]$/.test(event.key);
    if (typedSpecialCharacter && event.code.startsWith('Key')) return event.code.slice(3).toLowerCase();
    if (typedSpecialCharacter && event.code.startsWith('Digit')) return event.code.slice(5);
    return event.key.length === 1 ? event.key.toLowerCase() : event.key;
}

function isShiftedSymbol(keyName: string) {
    return keyName.length === 1 && !/[a-z0-9]/.test(keyName);
}

export function keyFromEvent(event: KeyboardEvent) {
    if (modifierKeys.includes(event.key)) return null;

    const modifiers: string[] = [];
    if (isMac ? event.metaKey : event.ctrlKey) modifiers.push('Mod');
    if (isMac && event.ctrlKey) modifiers.push('Ctrl');
    if (event.altKey) modifiers.push('Alt');

    const keyName = keyNameFromEvent(event);
    if (event.shiftKey && !isShiftedSymbol(keyName)) modifiers.push('Shift');

    const hasCommandModifier = modifiers.some((modifier) => modifier !== 'Shift');
    if (!hasCommandModifier && !/^F\d{1,2}$/.test(keyName)) return null;

    return [...modifiers, keyName].join('-');
}

export function matchesKeybind(key: string, event: KeyboardEvent) {
    const pressed = keyFromEvent(event);
    return Boolean(key && pressed && normalizeKey(pressed) === normalizeKey(key));
}

export function keyLabels(key: string) {
    if (!key) return [];
    const { modifiers, keyName } = splitKey(key);
    const sorted = modifierOrder.filter((modifier) => modifiers.includes(modifier));
    const keyLabel = keyName === 'Space' ? 'Space' : keyName.length === 1 ? keyName.toUpperCase() : keyName;
    return [...sorted.map((modifier) => modifierLabels[modifier]), keyLabel];
}

export function shortcutText(key: string) {
    return keyLabels(key).join('+');
}
