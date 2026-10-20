export interface Package {
    id: string;
    owner_id: string;
    name: string;
    description?: string;
    owner_name?: string;
    latest_version?: string;
    visibility: PackageVisibility;
    created_at: string;
}

export type PackageVisibility = 'public' | 'private';

export interface PackageVersion {
    id: string;
    version: string;
    entrypoint: string;
    created_at: string;
}

export interface PackagePolicy {
    publishing_enabled: boolean;
    admins_only: boolean;
    max_package_size_bytes: number;
    max_versions_per_package: number;
    allow_owner_delete: boolean;
    allow_private_packages: boolean;
    default_visibility: PackageVisibility;
}

export interface UserPackagePolicy extends PackagePolicy {
    can_publish: boolean;
    can_delete_own: boolean;
    can_publish_private: boolean;
}

export interface PackageAccess {
    visibility: PackageVisibility;
    users: { id: string; username: string; email?: string }[];
    teams: { id: string; name: string }[];
}

async function accessRequest(name: string, path: string, method: string, body?: unknown): Promise<PackageAccess> {
    const res = await fetch(`/api/packages/${encodeURIComponent(name)}/access${path}`, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text()) || 'Request failed');
    return res.json();
}

export const fetchPackageAccess = (name: string) => accessRequest(name, '', 'GET');
export const setPackageVisibility = (name: string, visibility: PackageVisibility) =>
    accessRequest(name, '/visibility', 'PUT', { visibility });
export const addPackageUser = (name: string, email: string) => accessRequest(name, '/users', 'POST', { email });
export const removePackageUser = (name: string, userId: string) => accessRequest(name, `/users/${userId}`, 'DELETE');
export const addPackageTeam = (name: string, teamId: string) => accessRequest(name, '/teams', 'POST', { team_id: teamId });
export const removePackageTeam = (name: string, teamId: string) => accessRequest(name, `/teams/${teamId}`, 'DELETE');

export async function fetchUserPackagePolicy(): Promise<UserPackagePolicy | null> {
    try {
        const res = await fetch('/api/packages/policy');
        return res.ok ? await res.json() : null;
    } catch {
        return null;
    }
}
