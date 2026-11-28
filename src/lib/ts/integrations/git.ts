export type GitProvider = 'auto' | 'github' | 'gitlab' | 'generic';
export type GitResolution = 'ask' | 'keep_repository' | 'keep_mine';

export type GitPolicy = {
    enabled: boolean;
    generic_hosts_enabled: boolean;
};

export type GitLink = {
    provider: Exclude<GitProvider, 'auto'>;
    repo_url: string;
    branch: string;
    folder: string;
    username: string | null;
    has_token: boolean;
    account_username: string | null;
    excluded_paths: string[];
    base_commit: string | null;
    last_synced_at: string | null;
    created_by_name: string | null;
    can_manage: boolean;
};

export type GitLinkStatus = GitPolicy & { link: GitLink | null };

export type GitLinkRequest = {
    provider: GitProvider;
    repo_url: string;
    branch: string;
    folder: string;
    username: string;
    token: string;
    excluded_paths: string[];
    replace_project_files: boolean;
};

export type GitPreviewFile = { path: string; size: number | null };

export type GitImportPreview = {
    commit: string;
    files: GitPreviewFile[];
    too_large: GitPreviewFile[];
    ignored_count: number;
    total_bytes: number;
    needed_bytes: number;
    available_bytes: number | null;
    replaced: string[];
    removed: string[];
};

export type GitChange = { path: string; change: 'added' | 'modified' | 'deleted' };

export type GitChanges = {
    link: GitLink;
    head: string;
    behind: boolean;
    local: GitChange[];
    remote: GitChange[];
    conflicts: string[];
};

export type GitPullResult = {
    link: GitLink;
    updated: string[];
    removed: string[];
    conflicts: string[];
    too_large: string[];
    applied: boolean;
};

export type GitPushResult = { link: GitLink; commit: string; pushed: string[] };

export type GitLinkUpdate = { token?: string; use_account?: boolean; username?: string; excluded_paths?: string[] };

export type GitAccountProvider = 'github' | 'gitlab';

export type GitAccount = {
    id: string;
    provider: GitAccountProvider;
    base_url: string;
    username: string;
    token_hint: string | null;
    created_at: string;
};

export const providerLabels: Record<GitProvider, string> = {
    auto: 'Detect from address',
    github: 'GitHub',
    gitlab: 'GitLab',
    generic: 'Other Git server'
};

async function request<T>(url: string, method = 'GET', body?: unknown): Promise<T> {
    const res = await fetch(url, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Request failed');
    return res.status === 204 ? (undefined as T) : res.json();
}

const projectBase = (projectId: string) => `/api/projects/${projectId}/git`;

export const fetchGitStatus = (projectId: string) => request<GitLinkStatus>(projectBase(projectId));
export const previewGitLink = (projectId: string, link: GitLinkRequest) =>
    request<GitImportPreview>(`${projectBase(projectId)}/preview`, 'POST', link);
export const createGitLink = (projectId: string, link: GitLinkRequest) => request<GitLinkStatus>(projectBase(projectId), 'POST', link);
export const updateGitLink = (projectId: string, update: GitLinkUpdate) => request<GitLink>(projectBase(projectId), 'PATCH', update);
export const deleteGitLink = (projectId: string) => request<void>(projectBase(projectId), 'DELETE');
export const fetchGitChanges = (projectId: string) => request<GitChanges>(`${projectBase(projectId)}/changes`);
export const pullGit = (projectId: string, resolution: GitResolution = 'ask') =>
    request<GitPullResult>(`${projectBase(projectId)}/pull`, 'POST', { resolution });
export const pushGit = (projectId: string, message: string, paths: string[]) =>
    request<GitPushResult>(`${projectBase(projectId)}/push`, 'POST', { message, paths });

export const fetchGitPolicy = () => request<GitPolicy>('/api/git/policy');
export const previewGitImport = (link: GitLinkRequest) => request<GitImportPreview>('/api/git/preview', 'POST', link);
export const importGitProject = (name: string, folderId: string | null, link: GitLinkRequest) =>
    request<{ id: string }>('/api/git/import', 'POST', { ...link, name, folder_id: folderId });

export const fetchGitAccounts = () => request<GitAccount[]>('/api/git/accounts');
export const connectGitAccount = (provider: GitAccountProvider, server: string, token: string) =>
    request<GitAccount[]>('/api/git/accounts', 'POST', { provider, server, token });
export const deleteGitAccount = (accountId: string) => request<void>(`/api/git/accounts/${accountId}`, 'DELETE');

export const fetchGitAdminPolicy = () => request<GitPolicy>('/api/admin/git-policy');
export const saveGitAdminPolicy = (policy: GitPolicy) => request<GitPolicy>('/api/admin/git-policy', 'PUT', policy);

export function shortCommit(commit: string | null): string {
    return commit ? commit.slice(0, 7) : '';
}

export function repositoryName(repoUrl: string): string {
    const segments = repoUrl.trim().replace(/\/+$/, '').replace(/\.git$/, '').split('/');
    return segments.length > 3 ? segments[segments.length - 1] : '';
}

function serverOf(repoUrl: string): string | null {
    try {
        const url = new URL(repoUrl.trim());
        return url.protocol === 'https:' || url.protocol === 'http:' ? `${url.protocol}//${url.host}`.toLowerCase() : null;
    } catch {
        return null;
    }
}

export function matchingAccount(accounts: GitAccount[], provider: GitProvider, repoUrl: string): GitAccount | null {
    const server = serverOf(repoUrl);
    if (!server) return null;
    const host = new URL(server).host;
    const resolved = provider !== 'auto' ? provider : host === 'github.com' ? 'github' : host === 'gitlab.com' ? 'gitlab' : null;
    return accounts.find((account) => account.provider === resolved && account.base_url === server) ?? null;
}

export function tokenPageUrl(account: Pick<GitAccount, 'provider' | 'base_url'>): string {
    return account.provider === 'github'
        ? `${account.base_url}/settings/personal-access-tokens/new`
        : `${account.base_url}/-/user_settings/personal_access_tokens`;
}

export function parseExcludedPaths(text: string): string[] {
    return text
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0);
}
