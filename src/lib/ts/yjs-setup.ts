import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { localAwarenessUser } from './awareness-user';
import { connectionStatus } from './store';
import { clearPresence, publishPresence, type PresenceSource } from './presence';

export let doc: Y.Doc | null = null;
export let text: Y.Text | null = null;
export let provider: WebsocketProvider | null = null;
export let undoManager: Y.UndoManager | null = null;

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

	provider.awareness.setLocalStateField('user', localAwarenessUser());

	provider.on('status', (event: { status: string }) => {
		connectionStatus.set(event.status);
		console.log(`Yjs connection status for ${docId}:`, event.status);
	});

	provider.awareness.on('change', () => publishPresence(presenceSources()));
}

export function presenceSources(): PresenceSource[] {
	return provider && doc ? [{ fileId: null, doc, awareness: provider.awareness }] : [];
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
	clearPresence();
}

