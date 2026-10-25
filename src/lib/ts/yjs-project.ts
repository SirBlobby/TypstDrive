import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { localAwarenessUser } from './awareness-user';
import { connectionStatus } from './store';
import { clearPresence, publishPresence, type PresenceSource } from './presence';

export interface OpenFile {
	fileId: string;
	path: string;
	doc: Y.Doc;
	text: Y.Text;
	provider: WebsocketProvider;
}

const open = new Map<string, OpenFile>();
let projectId: string | null = null;
let activeFileId: string | null = null;

const TEXT_NAME = 'typst';

export function setProject(id: string) {
	projectId = id;
}

export function openFile(fileId: string, path: string): OpenFile {
	const existing = open.get(fileId);
	if (existing) return existing;
	if (!projectId) throw new Error('Project not set');

	const doc = new Y.Doc();
	const text = doc.getText(TEXT_NAME);

	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const host = window.location.host;

	connectionStatus.set('connecting');

	const provider = new WebsocketProvider(`${protocol}//${host}/yjs`, `project:${projectId}:${fileId}`, doc);

	provider.awareness.setLocalStateField('user', localAwarenessUser());
	provider.awareness.setLocalStateField('activeFile', activeFileId);

	provider.on('status', (event: { status: string }) => {
		connectionStatus.set(event.status);
	});

	provider.awareness.on('change', () => publishPresence(presenceSources()));

	const entry: OpenFile = { fileId, path, doc, text, provider };
	open.set(fileId, entry);
	return entry;
}

export function presenceSources(): PresenceSource[] {
	return [...open.values()].map((entry) => ({ fileId: entry.fileId, doc: entry.doc, awareness: entry.provider.awareness }));
}

export function setActiveFile(fileId: string | null) {
	activeFileId = fileId;
	for (const entry of open.values()) entry.provider.awareness.setLocalStateField('activeFile', fileId);
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
		publishPresence(presenceSources());
	}
}

export function getAllText(): Record<string, string> {
	const result: Record<string, string> = {};
	for (const entry of open.values()) {
		result[entry.path] = entry.text.toString();
	}
	return result;
}

export function cleanupProject() {
	for (const fileId of Array.from(open.keys())) {
		closeFile(fileId);
	}
	projectId = null;
	activeFileId = null;
	connectionStatus.set('disconnected');
	clearPresence();
}
