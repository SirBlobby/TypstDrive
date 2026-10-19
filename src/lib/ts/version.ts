export const RELEASES_URL = 'https://github.com/SirBlobby/TypstDrive/releases';

const LATEST_RELEASE_API = 'https://api.github.com/repos/SirBlobby/TypstDrive/releases/latest';
const LATEST_RELEASE_CACHE_KEY = 'typstdrive-latest-release';
const CACHE_DURATION_MS = 60 * 60 * 1000;

export type LatestRelease = {
    version: string;
    url: string;
};

type CachedRelease = LatestRelease & { checkedAt: number };

export async function fetchServerVersion(): Promise<string | null> {
    try {
        const res = await fetch('/api/desktop/version');
        if (!res.ok) return null;
        const info = await res.json();
        return info.server_version ?? null;
    } catch {
        return null;
    }
}

function readCachedRelease(): LatestRelease | null {
    try {
        const raw = sessionStorage.getItem(LATEST_RELEASE_CACHE_KEY);
        if (!raw) return null;
        const cached: CachedRelease = JSON.parse(raw);
        if (Date.now() - cached.checkedAt > CACHE_DURATION_MS) return null;
        return { version: cached.version, url: cached.url };
    } catch {
        return null;
    }
}

function writeCachedRelease(release: LatestRelease) {
    try {
        const cached: CachedRelease = { ...release, checkedAt: Date.now() };
        sessionStorage.setItem(LATEST_RELEASE_CACHE_KEY, JSON.stringify(cached));
    } catch {}
}

export async function fetchLatestRelease(): Promise<LatestRelease | null> {
    const cached = readCachedRelease();
    if (cached) return cached;

    try {
        const res = await fetch(LATEST_RELEASE_API, { headers: { Accept: 'application/vnd.github+json' } });
        if (!res.ok) return null;
        const release = await res.json();
        if (typeof release.tag_name !== 'string') return null;

        const latest: LatestRelease = {
            version: release.tag_name.replace(/^v/i, ''),
            url: release.html_url ?? RELEASES_URL
        };
        writeCachedRelease(latest);
        return latest;
    } catch {
        return null;
    }
}

function versionParts(version: string): number[] {
    return version
        .split('-')[0]
        .split('.')
        .map((part) => parseInt(part, 10) || 0);
}

export function isNewerVersion(candidate: string, current: string): boolean {
    const candidateParts = versionParts(candidate);
    const currentParts = versionParts(current);
    const length = Math.max(candidateParts.length, currentParts.length);

    for (let index = 0; index < length; index++) {
        const candidatePart = candidateParts[index] ?? 0;
        const currentPart = currentParts[index] ?? 0;
        if (candidatePart !== currentPart) return candidatePart > currentPart;
    }
    return false;
}
