import type { TemplateAuthor } from './orcid';

export interface UniverseTemplate {
    name: string;
    version: string;
    description: string;
    authors: string[];
    categories: string[];
    disciplines: string[];
    keywords: string[];
    has_thumbnail: boolean;
    updated_at: number | null;
}

export interface CreatedItem {
    kind: 'document' | 'project';
    id: string;
    authors_filled?: boolean;
}

export type TemplateSort = 'name' | 'recent';

const categoryLabels: Record<string, string> = {
    cv: 'CV',
    paper: 'Paper',
    thesis: 'Thesis',
    report: 'Report',
    presentation: 'Presentation',
    poster: 'Poster',
    flyer: 'Flyer',
    book: 'Book',
    office: 'Office',
    fun: 'Fun'
};

async function request<T>(url: string, method = 'GET', body?: unknown): Promise<T> {
    const res = await fetch(url, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Request failed');
    return res.json();
}

export const fetchTemplates = () => request<UniverseTemplate[]>('/api/templates');

export const createFromTemplate = (template: UniverseTemplate, title: string, folderId: string | null, authors: TemplateAuthor[]) =>
    request<CreatedItem>('/api/projects/from-template', 'POST', {
        name: template.name,
        version: template.version,
        title,
        folder_id: folderId ?? undefined,
        authors
    });

export function thumbnailUrl(template: UniverseTemplate) {
    return `/api/templates/${encodeURIComponent(template.name)}/${encodeURIComponent(template.version)}/thumbnail`;
}

export function universeUrl(template: UniverseTemplate) {
    return `https://typst.app/universe/package/${encodeURIComponent(template.name)}`;
}

export function createdItemUrl(item: CreatedItem) {
    return item.kind === 'project' ? `/project/${item.id}` : `/doc/${item.id}`;
}

export function categoryLabel(category: string) {
    return categoryLabels[category] ?? category.charAt(0).toUpperCase() + category.slice(1).replace(/-/g, ' ');
}

export function readableName(name: string) {
    return name
        .split('-')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
}

export function matchesQuery(template: UniverseTemplate, query: string) {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length === 0) return true;
    const haystack = [template.name, template.description, ...template.keywords, ...template.authors, ...template.categories, ...template.disciplines]
        .join(' ')
        .toLowerCase();
    return words.every((word) => haystack.includes(word));
}

export function sortTemplates(templates: UniverseTemplate[], sort: TemplateSort) {
    const sorted = [...templates];
    if (sort === 'recent') sorted.sort((left, right) => (right.updated_at ?? 0) - (left.updated_at ?? 0));
    else sorted.sort((left, right) => left.name.localeCompare(right.name));
    return sorted;
}

export function categoryCounts(templates: UniverseTemplate[]) {
    const counts = new Map<string, number>();
    for (const template of templates) {
        for (const category of template.categories) counts.set(category, (counts.get(category) ?? 0) + 1);
    }
    return [...counts.entries()].sort((left, right) => right[1] - left[1]);
}
