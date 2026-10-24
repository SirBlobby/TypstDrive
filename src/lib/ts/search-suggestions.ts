import { isValidTokenValue, parseTokenDraft, searchFilters, type SearchToken } from './drive-search';

export type SuggestionAction =
    | { kind: 'draft'; text: string }
    | { kind: 'token'; token: SearchToken }
    | { kind: 'hint' };

export interface SearchSuggestion {
    label: string;
    description: string;
    icon: string;
    action: SuggestionAction;
}

function todayDate() {
    const now = new Date();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    return `${now.getFullYear()}-${month}-${day}`;
}

function filterKeySuggestions(text: string): SearchSuggestion[] {
    const typed = text.trim().toLowerCase();
    if (typed.includes(' ')) return [];
    return searchFilters
        .filter((filter) => filter.key.startsWith(typed))
        .map((filter) => ({
            label: `${filter.key}:`,
            description: filter.description,
            icon: filter.icon,
            action: { kind: 'draft', text: `${filter.key}:` }
        }));
}

export function searchSuggestions(text: string): SearchSuggestion[] {
    const draft = parseTokenDraft(text);
    if (!draft) return filterKeySuggestions(text);

    const { filter, value } = draft;
    const hint: SearchSuggestion = { label: `${filter.key}: ${filter.hint}`, description: filter.description, icon: filter.icon, action: { kind: 'hint' } };

    if (filter.values) {
        const matching = filter.values.filter((option) => option.value.startsWith(value));
        if (matching.length === 0) return [hint];
        return matching.map((option) => ({
            label: `${filter.key}: ${option.value}`,
            description: option.description,
            icon: filter.icon,
            action: { kind: 'token', token: { key: filter.key, value: option.value } }
        }));
    }

    if (isValidTokenValue(filter, value)) {
        const label = filter.key === 'doc' ? `Search inside documents for "${value}"` : `${filter.key}: ${value}`;
        return [{ label, description: filter.description, icon: filter.icon, action: { kind: 'token', token: { key: filter.key, value } } }];
    }

    if (filter.isDate) {
        const today = todayDate();
        return [hint, { label: `${filter.key}: ${today}`, description: 'Today', icon: filter.icon, action: { kind: 'token', token: { key: filter.key, value: today } } }];
    }

    return [hint];
}
