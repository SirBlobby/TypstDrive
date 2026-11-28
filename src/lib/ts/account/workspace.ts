import { get, writable } from 'svelte/store';

export type TeamRole = 'owner' | 'admin' | 'member' | 'viewer';

export type TeamSummary = {
    id: string;
    name: string;
    role: TeamRole;
    member_count: number;
    created_at: string;
};

export type TeamMember = {
    user_id: string;
    username: string;
    email: string;
    role: TeamRole;
    created_at: string;
};

export type TeamDetail = TeamSummary & {
    members: TeamMember[];
    max_members: number;
};

export type TeamPolicy = {
    enabled: boolean;
    creation: 'everyone' | 'admins';
    max_teams_per_user: number;
    max_members_per_team: number;
    can_create: boolean;
    owned_teams: number;
};

const STORAGE_KEY = 'typstdrive-workspace';
const SCOPED_PAGES = ['/drive', '/packages'];

function savedWorkspace(): TeamSummary | null {
    if (typeof window === 'undefined') return null;
    try {
        return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null');
    } catch {
        return null;
    }
}

export const activeTeamStore = writable<TeamSummary | null>(savedWorkspace());
export const teamsStore = writable<TeamSummary[]>([]);

activeTeamStore.subscribe((team) => {
    if (typeof window === 'undefined') return;
    if (team) localStorage.setItem(STORAGE_KEY, JSON.stringify(team));
    else localStorage.removeItem(STORAGE_KEY);
});

function requestPath(input: RequestInfo | URL): string | null {
    const raw = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;
    const url = new URL(raw, window.location.origin);
    return url.origin === window.location.origin ? url.pathname : null;
}

let headerInstalled = false;

export function installWorkspaceHeader() {
    if (headerInstalled || typeof window === 'undefined') return;
    headerInstalled = true;
    const originalFetch = window.fetch.bind(window);
    window.fetch = (input: RequestInfo | URL, init?: RequestInit) => {
        const team = get(activeTeamStore);
        const path = requestPath(input);
        const onScopedPage = SCOPED_PAGES.some((page) => window.location.pathname.startsWith(page));
        if (!team || !path?.startsWith('/api/') || !onScopedPage) return originalFetch(input, init);

        const headers = new Headers(init?.headers ?? (input instanceof Request ? input.headers : undefined));
        headers.set('X-Workspace', team.id);
        return originalFetch(input, { ...init, headers });
    };
}

async function requestJson<T>(url: string, method = 'GET', body?: unknown): Promise<T> {
    const res = await fetch(url, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text()) || 'Request failed');
    return res.status === 204 ? (undefined as T) : res.json();
}

export async function refreshTeams(): Promise<TeamSummary[]> {
    try {
        const teams = await requestJson<TeamSummary[]>('/api/teams');
        teamsStore.set(teams);
        const active = get(activeTeamStore);
        if (active) activeTeamStore.set(teams.find((team) => team.id === active.id) ?? null);
        return teams;
    } catch {
        teamsStore.set([]);
        return [];
    }
}

export const fetchTeamPolicy = () => requestJson<TeamPolicy>('/api/teams/policy');
export const createTeam = (name: string) => requestJson<TeamSummary>('/api/teams', 'POST', { name });
export const fetchTeam = (id: string) => requestJson<TeamDetail>(`/api/teams/${id}`);
export const renameTeam = (id: string, name: string) => requestJson<TeamSummary>(`/api/teams/${id}`, 'PATCH', { name });
export const deleteTeam = (id: string) => requestJson<void>(`/api/teams/${id}`, 'DELETE');
export const addTeamMember = (id: string, email: string, role: TeamRole) =>
    requestJson<TeamMember[]>(`/api/teams/${id}/members`, 'POST', { email, role });
export const changeTeamRole = (id: string, userId: string, role: TeamRole) =>
    requestJson<TeamMember[]>(`/api/teams/${id}/members/${userId}`, 'PATCH', { role });
export const removeTeamMember = (id: string, userId: string) => requestJson<void>(`/api/teams/${id}/members/${userId}`, 'DELETE');
export const transferTeam = (id: string, userId: string) =>
    requestJson<TeamMember[]>(`/api/teams/${id}/members/${userId}/transfer`, 'POST');

export const ROLE_LABELS: Record<TeamRole, string> = {
    owner: 'Owner',
    admin: 'Admin',
    member: 'Member',
    viewer: 'Viewer'
};

export const ROLE_DESCRIPTIONS: Record<Exclude<TeamRole, 'owner'>, string> = {
    admin: 'Can manage members and all content',
    member: 'Can create, edit and delete projects and documents',
    viewer: 'Can open content but not change it'
};
