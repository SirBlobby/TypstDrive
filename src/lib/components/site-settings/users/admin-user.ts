export type AdminUser = {
    id: string;
    username: string;
    email: string;
    is_admin: boolean;
    email_verified: boolean;
    created_at: string;
    two_factor_method: 'totp' | 'email' | null;
    password_change_required: boolean;
};

export type IssuedPassword = {
    username: string;
    password: string;
    expires_at: string;
};

export function formatExpiry(value: string) {
    return new Date(`${value.replace(' ', 'T')}Z`).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
}
