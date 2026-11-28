export interface FuzzyMatch {
    score: number;
    matchedIndexes: number[];
}

const PREFIX_BONUS = 30;
const WORD_START_BONUS = 10;
const CONSECUTIVE_BONUS = 6;
const BASE_POINTS = 1;
const GAP_PENALTY = 0.5;

function isWordStart(text: string, index: number) {
    return index === 0 || /[\s\-_./(]/.test(text[index - 1]);
}

export function fuzzyMatch(query: string, text: string): FuzzyMatch | null {
    const needle = query.toLowerCase();
    const haystack = text.toLowerCase();
    if (!needle) return { score: 0, matchedIndexes: [] };

    const matchedIndexes: number[] = [];
    let score = haystack.startsWith(needle) ? PREFIX_BONUS : 0;
    let searchFrom = 0;
    let previousIndex = -2;

    for (const character of needle) {
        const index = haystack.indexOf(character, searchFrom);
        if (index === -1) return null;
        score += BASE_POINTS;
        if (isWordStart(text, index)) score += WORD_START_BONUS;
        if (index === previousIndex + 1) score += CONSECUTIVE_BONUS;
        else if (previousIndex >= 0) score -= (index - previousIndex - 1) * GAP_PENALTY;
        matchedIndexes.push(index);
        previousIndex = index;
        searchFrom = index + 1;
    }
    return { score, matchedIndexes };
}

export function bestMatch(query: string, label: string, keywords: string[]) {
    const labelMatch = fuzzyMatch(query, label);
    let best = labelMatch ? { ...labelMatch, onLabel: true } : null;
    for (const keyword of keywords) {
        const keywordMatch = fuzzyMatch(query, keyword);
        if (keywordMatch && (!best || keywordMatch.score > best.score)) best = { score: keywordMatch.score, matchedIndexes: [], onLabel: false };
    }
    return best;
}
