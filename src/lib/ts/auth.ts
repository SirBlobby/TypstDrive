import { writable } from 'svelte/store';

export type User = {
    id: string;
    username: string;
    email: string;
    is_admin: boolean;
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
