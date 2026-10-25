import { writable } from 'svelte/store';
import type * as Y from 'yjs';
import type { Awareness } from 'y-protocols/awareness';
import { connectedUsers, type AwarenessUser } from './store';

export interface PresenceSource {
	fileId: string | null;
	doc: Y.Doc;
	awareness: Awareness;
}

export const filePresenceStore = writable<Map<string, AwarenessUser[]>>(new Map());

function personKey(user: AwarenessUser) {
	return user.id ?? user.name;
}

function addUnique(users: Map<string, AwarenessUser>, user: AwarenessUser) {
	const key = personKey(user);
	const existing = users.get(key);
	if (!existing || (user.isLocal && !existing.isLocal)) users.set(key, user);
}

export function collectPresence(sources: PresenceSource[]) {
	const everyone = new Map<string, AwarenessUser>();
	const byFile = new Map<string, Map<string, AwarenessUser>>();

	for (const source of sources) {
		const localId = source.awareness.clientID;
		source.awareness.getStates().forEach((state, clientId) => {
			if (!state.user) return;
			const user: AwarenessUser = {
				clientId,
				...state.user,
				activeFile: state.activeFile ?? source.fileId,
				isLocal: clientId === localId
			};
			addUnique(everyone, user);
			if (user.activeFile && !user.isLocal) {
				const inFile = byFile.get(user.activeFile) ?? new Map<string, AwarenessUser>();
				addUnique(inFile, user);
				byFile.set(user.activeFile, inFile);
			}
		});
	}

	return {
		users: [...everyone.values()],
		byFile: new Map([...byFile.entries()].map(([fileId, users]) => [fileId, [...users.values()]]))
	};
}

export function publishPresence(sources: PresenceSource[]) {
	const { users, byFile } = collectPresence(sources);
	connectedUsers.set(users);
	filePresenceStore.set(byFile);
}

export function clearPresence() {
	connectedUsers.set([]);
	filePresenceStore.set(new Map());
}
