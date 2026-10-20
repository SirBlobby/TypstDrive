const browsers: [RegExp, string][] = [
    [/Edg\//, 'Edge'],
    [/OPR\//, 'Opera'],
    [/Firefox\//, 'Firefox'],
    [/Chrome\//, 'Chrome'],
    [/Safari\//, 'Safari']
];

const systems: [RegExp, string][] = [
    [/Windows/, 'Windows'],
    [/iPhone|iPad/, 'iOS'],
    [/Android/, 'Android'],
    [/Mac OS X|Macintosh/, 'macOS'],
    [/Linux/, 'Linux']
];

function firstMatch(text: string, patterns: [RegExp, string][]): string | null {
    return patterns.find(([pattern]) => pattern.test(text))?.[1] ?? null;
}

export function describeUserAgent(userAgent: string | null | undefined): string {
    if (!userAgent) return 'Unknown device';
    const browser = firstMatch(userAgent, browsers);
    const system = firstMatch(userAgent, systems);
    if (browser && system) return `${browser} on ${system}`;
    return browser ?? system ?? 'Unknown device';
}

export function userAgentIcon(userAgent: string | null | undefined): string {
    if (!userAgent) return 'mdi:devices';
    return /iPhone|Android|Mobile/.test(userAgent) ? 'mdi:cellphone' : 'mdi:monitor';
}
