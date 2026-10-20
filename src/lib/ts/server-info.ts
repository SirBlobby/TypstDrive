export interface EnvironmentVariable {
    name: string;
    group: string;
    description: string;
    is_set: boolean;
}

export interface ServerInfo {
    server_version: string;
    database: {
        kind: string;
        version: string | null;
        schema_version: number;
        max_connections: number;
    };
    realtime: {
        redis_enabled: boolean;
        connected: boolean;
        instance_id: string;
    };
    file_storage: {
        object_storage_enabled: boolean;
        files_in_database: number;
        files_in_object_storage: number;
        cache_limit_bytes: number;
    };
    environment: EnvironmentVariable[];
}

export async function fetchServerInfo(): Promise<ServerInfo> {
    const res = await fetch('/api/admin/server');
    if (!res.ok) throw new Error((await res.text()) || 'Failed to load server details.');
    return res.json();
}
