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
