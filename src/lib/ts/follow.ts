import * as Y from 'yjs';
import { EditorView } from '@codemirror/view';
import { get, writable } from 'svelte/store';
import type { Awareness } from 'y-protocols/awareness';
import { editorViewStore } from './store';
import type { PresenceSource } from './presence';

export const followingStore = writable<string | null>(null);

const STOP_EVENTS = ['mousedown', 'keydown', 'wheel', 'touchstart'] as const;

export function toggleFollow(session: string) {
	followingStore.update((current) => (current === session ? null : session));
}

export function stopFollowing() {
	followingStore.set(null);
}

interface FollowOptions {
	sources: () => PresenceSource[];
	activeFileId?: () => string;
	openFile?: (fileId: string) => void;
}

function findTarget(sources: PresenceSource[], session: string) {
	for (const source of sources) {
		for (const state of source.awareness.getStates().values()) {
			if (state.user?.session === session) return { activeFile: (state.activeFile as string | null | undefined) ?? null };
		}
	}
	return null;
}

function cursorHead(source: PresenceSource, session: string) {
	for (const [clientId, state] of source.awareness.getStates()) {
		if (clientId === source.awareness.clientID || state.user?.session !== session || !state.cursor?.head) continue;
		const absolute = Y.createAbsolutePositionFromRelativePosition(Y.createRelativePositionFromJSON(state.cursor.head), source.doc);
		return absolute ? absolute.index : null;
	}
	return null;
}

export function createFollowController(options: FollowOptions) {
	const watched = new Set<Awareness>();
	let view: EditorView | null = null;

	function update() {
		const session = get(followingStore);
		const sources = options.sources();
		for (const source of sources) watch(source.awareness);
		if (!session) return;

		const target = findTarget(sources, session);
		if (!target) {
			stopFollowing();
			return;
		}

		const targetFile = target.activeFile;
		if (targetFile && options.activeFileId && options.openFile && targetFile !== options.activeFileId()) {
			options.openFile(targetFile);
			return;
		}

		const source = sources.find((candidate) => !targetFile || candidate.fileId === null || candidate.fileId === targetFile);
		const position = source ? cursorHead(source, session) : null;
		if (view && position !== null) {
			const clamped = Math.min(position, view.state.doc.length);
			view.dispatch({ effects: EditorView.scrollIntoView(clamped, { y: 'center' }) });
		}
	}

	function watch(awareness: Awareness) {
		if (watched.has(awareness)) return;
		watched.add(awareness);
		awareness.on('change', update);
	}

	function attachView(next: EditorView | null) {
		if (view) STOP_EVENTS.forEach((name) => view?.dom.removeEventListener(name, stopFollowing));
		view = next;
		if (view) STOP_EVENTS.forEach((name) => view?.dom.addEventListener(name, stopFollowing));
		queueMicrotask(update);
	}

	const unsubscribeView = editorViewStore.subscribe(attachView);
	const unsubscribeFollow = followingStore.subscribe(() => queueMicrotask(update));

	return () => {
		unsubscribeView();
		unsubscribeFollow();
		attachView(null);
		watched.forEach((awareness) => awareness.off('change', update));
		watched.clear();
		stopFollowing();
	};
}
