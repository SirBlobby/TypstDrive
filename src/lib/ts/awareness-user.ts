import { get } from 'svelte/store';
import { userStore } from './auth';

const userColors = [
	'#30bced', '#6eeb83', '#ffbc42', '#ecd444', '#ee6352',
	'#9ac2c9', '#8acb88', '#1be7ff', '#ff0054', '#9e0059'
];

function colorFor(seed: string) {
	let hash = 0;
	for (const character of seed) hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
	return userColors[hash % userColors.length];
}

export function localAwarenessUser() {
	const user = get(userStore);
	const color = user ? colorFor(user.id) : userColors[Math.floor(Math.random() * userColors.length)];
	const avatar = user?.avatar_updated_at
		? `/api/users/${user.id}/avatar?v=${encodeURIComponent(user.avatar_updated_at)}`
		: null;
	return {
		id: user?.id ?? null,
		name: user?.username || 'Anonymous',
		color,
		colorLight: color + '33',
		avatar
	};
}
