export interface OrcidAffiliation {
    organization: string;
    department: string | null;
    role: string | null;
    city: string | null;
    country: string | null;
    current: boolean;
}

export interface OrcidProfile {
    orcid: string;
    name: string;
    given_names: string | null;
    family_name: string | null;
    email: string | null;
    affiliations: OrcidAffiliation[];
}

export interface TemplateAuthor {
    name: string;
    given_names?: string | null;
    family_name?: string | null;
    affiliation?: string | null;
    department?: string | null;
    email?: string | null;
    orcid?: string | null;
}

const SAVED_AUTHORS_KEY = 'typstdrive.template-authors';

export async function fetchOrcidProfile(orcid: string): Promise<OrcidProfile> {
    const res = await fetch(`/api/citations/orcid/${encodeURIComponent(orcid.trim())}`);
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Could not look up that ORCID iD');
    return res.json();
}

export function authorFromProfile(profile: OrcidProfile, affiliationIndex = 0): TemplateAuthor {
    const affiliation = profile.affiliations[affiliationIndex];
    return {
        name: profile.name,
        given_names: profile.given_names,
        family_name: profile.family_name,
        affiliation: affiliation?.organization ?? null,
        department: affiliation?.department ?? null,
        email: profile.email,
        orcid: profile.orcid
    };
}

export function affiliationLabel(affiliation: OrcidAffiliation) {
    return [affiliation.department, affiliation.organization].filter(Boolean).join(', ');
}

export function loadSavedAuthors(): TemplateAuthor[] {
    try {
        const saved = JSON.parse(localStorage.getItem(SAVED_AUTHORS_KEY) ?? '[]');
        return Array.isArray(saved) ? saved.filter((author) => typeof author?.name === 'string') : [];
    } catch {
        return [];
    }
}

export function saveAuthors(authors: TemplateAuthor[]) {
    try {
        localStorage.setItem(SAVED_AUTHORS_KEY, JSON.stringify(authors));
    } catch {}
}
