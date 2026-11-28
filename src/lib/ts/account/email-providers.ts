export interface EmailProviderGuide {
    value: string;
    label: string;
    credentials: string;
    region?: string;
}

export const emailProviderGuides: EmailProviderGuide[] = [
    { value: 'google', label: 'Google', credentials: 'Your Google address and an app password. Needs 2-Step Verification.' },
    { value: 'mailgun', label: 'Mailgun', credentials: 'An SMTP login and password from your sending domain.', region: 'us or eu' },
    { value: 'sendgrid', label: 'SendGrid', credentials: 'An API key as the password. The username is filled in for you.' },
    { value: 'ses', label: 'Amazon SES', credentials: 'SMTP credentials created in the SES console.', region: 'such as eu-west-1' },
    { value: 'resend', label: 'Resend', credentials: 'An API key as the password. The username is filled in for you.' },
    { value: 'mailjet', label: 'Mailjet', credentials: 'An API key as the username and its secret key as the password.' },
    { value: 'microsoft', label: 'Microsoft 365', credentials: 'A mailbox address and password with SMTP sending turned on.' },
    { value: 'fastmail', label: 'Fastmail', credentials: 'Your Fastmail address and an app password.' }
];
