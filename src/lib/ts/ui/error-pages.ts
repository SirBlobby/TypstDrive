export type ErrorCopy = {
	icon: string;
	title: string;
	description: string;
	canRetry: boolean;
};

const NOT_FOUND: ErrorCopy = {
	icon: 'mdi:file-search-outline',
	title: 'Page not found',
	description: 'This page does not exist, or it was moved or deleted. Check the address, or head back to your drive.',
	canRetry: false
};

const COPY_BY_STATUS: Record<number, ErrorCopy> = {
	400: {
		icon: 'mdi:alert-outline',
		title: 'That request did not look right',
		description: 'Something in the address or the request was not valid. Go back and try again.',
		canRetry: false
	},
	401: {
		icon: 'mdi:lock-outline',
		title: 'Sign in to continue',
		description: 'You need to be signed in to see this page.',
		canRetry: false
	},
	403: {
		icon: 'mdi:shield-lock-outline',
		title: 'You do not have access',
		description: 'This item was not shared with you, or your access was removed. Ask its owner to share it with you.',
		canRetry: false
	},
	404: NOT_FOUND,
	410: {
		icon: 'mdi:link-variant-off',
		title: 'This link has expired',
		description: 'The link you followed is no longer active. Ask the person who sent it for a new one.',
		canRetry: false
	},
	429: {
		icon: 'mdi:timer-sand',
		title: 'Too many requests',
		description: 'You are going a little fast. Wait a moment, then try again.',
		canRetry: true
	},
	503: {
		icon: 'mdi:cloud-off-outline',
		title: 'The server is busy',
		description: 'The server cannot answer right now. Try again in a moment.',
		canRetry: true
	}
};

const SERVER_ERROR: ErrorCopy = {
	icon: 'mdi:alert-circle-outline',
	title: 'Something went wrong',
	description: 'An unexpected error stopped this page from loading. Try again, and if it keeps happening, let your administrator know.',
	canRetry: true
};

export function errorCopy(status: number): ErrorCopy {
	return COPY_BY_STATUS[status] ?? (status >= 500 ? SERVER_ERROR : NOT_FOUND);
}
