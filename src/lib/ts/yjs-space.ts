import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { get } from 'svelte/store';
import { userStore } from './auth';
import { connectionStatus, connectedUsers } from './store';
import type { AwarenessUser } from './store';

export interface OpenFile {
	fileId: string;
	path: string;
	doc: Y.Doc;
	text: Y.Text;
	provider: WebsocketProvider;
}

const userColors = [
	'#30bced', '#6eeb83', '#ffbc42', '#ecd444', '#ee6352',
	'#9ac2c9', '#8acb88', '#1be7ff', '#ff0054', '#9e0059'
];

const open = new Map<string, OpenFile>();
let spaceId: string | null = null;

const TEXT_NAME = 'typst';

export function setSpace(id: string) {
	spaceId = id;
}

export function openFile(fileId: string, path: string): OpenFile {
	const existing = open.get(fileId);
	if (existing) return existing;
	if (!spaceId) throw new Error('Space not set');

	const doc = new Y.Doc();
	const text = doc.getText(TEXT_NAME);

	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const host = window.location.host;

	connectionStatus.set('connecting');

	const provider = new WebsocketProvider(`${protocol}//${host}/yjs`, `space:${spaceId}:${fileId}`, doc);

	const user = get(userStore);
	const color = userColors[Math.floor(Math.random() * userColors.length)];
	provider.awareness.setLocalStateField('user', {
		name: user?.username || 'Anonymous',
		color,
		colorLight: color + '33'
	});

	provider.on('status', (event: { status: string }) => {
		connectionStatus.set(event.status);
	});

	provider.awareness.on('change', () => {
		const states = provider.awareness.getStates();
		const localId = provider.awareness.clientID;
		const uniqueUsers = new Map<string, AwarenessUser>();
		states.forEach((state, clientId) => {
			if (state.user) {
				const isLocal = clientId === localId;
				const userObj = { clientId, ...state.user, isLocal };
				if (isLocal) {
					uniqueUsers.set(state.user.name, userObj);
				} else if (!uniqueUsers.has(state.user.name) || !uniqueUsers.get(state.user.name)!.isLocal) {
					uniqueUsers.set(state.user.name, userObj);
				}
			}
		});
		connectedUsers.set(Array.from(uniqueUsers.values()));
	});

	const entry: OpenFile = { fileId, path, doc, text, provider };
	open.set(fileId, entry);
	return entry;
}

export function getOpenFile(fileId: string): OpenFile | undefined {
	return open.get(fileId);
}

export function renameOpenFile(fileId: string, path: string) {
	const entry = open.get(fileId);
	if (entry) entry.path = path;
}

export function closeFile(fileId: string) {
	const entry = open.get(fileId);
	if (entry) {
		entry.provider.disconnect();
		entry.provider.destroy();
		entry.doc.destroy();
		open.delete(fileId);
	}
}

export function getAllText(): Record<string, string> {
	const result: Record<string, string> = {};
	for (const entry of open.values()) {
		result[entry.path] = entry.text.toString();
	}
	return result;
}

export function cleanupSpace() {
	for (const fileId of Array.from(open.keys())) {
		closeFile(fileId);
	}
	spaceId = null;
	connectionStatus.set('disconnected');
	connectedUsers.set([]);
}
