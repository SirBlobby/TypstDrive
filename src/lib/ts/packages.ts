export interface Package {
    id: string;
    owner_id: string;
    name: string;
    description?: string;
    owner_name?: string;
    latest_version?: string;
    created_at: string;
}

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
}

export interface UserPackagePolicy extends PackagePolicy {
    can_publish: boolean;
    can_delete_own: boolean;
}

export async function fetchUserPackagePolicy(): Promise<UserPackagePolicy | null> {
    try {
        const res = await fetch('/api/packages/policy');
        return res.ok ? await res.json() : null;
    } catch {
        return null;
    }
}
