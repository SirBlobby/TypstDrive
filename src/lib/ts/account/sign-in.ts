import type { User } from './auth';

export type TwoFactorMethod = 'totp' | 'email';

export type PendingStep = {
    next_step: 'change_password' | 'two_factor';
    method: TwoFactorMethod | null;
};

export type SignInResult = User | PendingStep;

export const CONTINUE_SIGN_IN_PATH = '/login?step=continue';

export function isPendingStep(result: SignInResult): result is PendingStep {
    return 'next_step' in result;
}

export async function fetchPendingStep(): Promise<PendingStep | null> {
    const res = await fetch('/api/auth/sign-in/pending');
    return res.ok ? res.json() : null;
}

export function continueSignInUrl(redirect: string): string {
    return `${CONTINUE_SIGN_IN_PATH}&redirect=${encodeURIComponent(redirect)}`;
}
