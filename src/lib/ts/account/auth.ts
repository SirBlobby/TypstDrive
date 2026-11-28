import { writable } from 'svelte/store';
import { activeTeamStore, teamsStore } from './workspace';

export type User = {
    id: string;
    username: string;
    email: string;
    is_admin: boolean;
    email_verified?: boolean;
    has_password?: boolean;
    avatar_updated_at?: string | null;
    two_factor_method?: 'totp' | 'email' | null;
};

export const userStore = writable<User | null>(null);

export async function fetchUser() {
    try {
        const res = await fetch('/api/auth/me');
        if (res.ok) {
            const user = await res.json();
            userStore.set(user);
        } else {
            userStore.set(null);
        }
    } catch {
        userStore.set(null);
    }
}

export async function signOut() {
    try {
        await fetch('/api/auth/logout', { method: 'POST' });
    } finally {
        activeTeamStore.set(null);
        teamsStore.set([]);
        userStore.set(null);
    }
}

export function avatarUrl(user: User | null): string | null {
    if (!user?.avatar_updated_at) return null;
    return `/api/auth/avatar?v=${encodeURIComponent(user.avatar_updated_at)}`;
}
