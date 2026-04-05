import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { get } from 'svelte/store';
import { userStore } from './auth';
import { connectionStatus, connectedUsers } from './store';
import type { AwarenessUser } from './store';

export let doc: Y.Doc | null = null;
export let text: Y.Text | null = null;
export let provider: WebsocketProvider | null = null;
export let undoManager: Y.UndoManager | null = null;

const userColors = [
	'#30bced', '#6eeb83', '#ffbc42', '#ecd444', '#ee6352',
	'#9ac2c9', '#8acb88', '#1be7ff', '#ff0054', '#9e0059'
];

export function initYjs(docId: string) {
	if (typeof window === 'undefined') return;
	
	
	if (provider) {
		provider.disconnect();
		provider = null;
	}

	doc = new Y.Doc();
	text = doc.getText('typst');
	undoManager = new Y.UndoManager(text);

	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const host = window.location.host;

	connectionStatus.set('connecting');

	provider = new WebsocketProvider(
		`${protocol}//${host}/yjs`,
		docId,
		doc
	);

	const user = get(userStore);
	const color = userColors[Math.floor(Math.random() * userColors.length)];
	
	provider.awareness.setLocalStateField('user', {
		name: user?.username || 'Anonymous',
		color: color,
		colorLight: color + '33'
	});

	provider.on('status', (event: { status: string }) => {
		connectionStatus.set(event.status);
		console.log(`Yjs connection status for ${docId}:`, event.status);
	});

	provider.awareness.on('change', () => {
		if (!provider) return;
		const states = provider.awareness.getStates();
		const localId = provider.awareness.clientID;
		
		
		
		const uniqueUsers = new Map<string, AwarenessUser>();
		
		states.forEach((state, clientId) => {
			if (state.user) {
				const isLocal = clientId === localId;
				const userObj = {
					clientId,
					...state.user,
					isLocal
				};
				
				if (isLocal) {
					
					uniqueUsers.set(state.user.name, userObj);
				} else if (!uniqueUsers.has(state.user.name) || !uniqueUsers.get(state.user.name)!.isLocal) {
					
					
					uniqueUsers.set(state.user.name, userObj);
				}
			}
		});
		
		connectedUsers.set(Array.from(uniqueUsers.values()));
	});
}

export function cleanupYjs() {
	if (provider) {
		provider.disconnect();
		provider = null;
	}
	if (undoManager) {
		undoManager.destroy();
		undoManager = null;
	}
	doc = null;
	text = null;
	connectionStatus.set('disconnected');
	connectedUsers.set([]);
}

