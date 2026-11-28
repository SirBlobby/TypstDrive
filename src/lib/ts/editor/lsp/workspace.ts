import { LSPPlugin, Workspace, type LSPClient, type WorkspaceFile } from '@codemirror/lsp-client';
import { ChangeSet, Text } from '@codemirror/state';
import type { EditorView } from '@codemirror/view';
import type * as Y from 'yjs';

export type TrackedText = { path: string; text: Y.Text };

export type WorkspaceHooks = {
	files: () => Iterable<TrackedText>;
	openPath?: (path: string) => void;
	scheduleSync: () => void;
};

const DISPLAY_TIMEOUT_MS = 3000;

export function languageIdFor(path: string) {
	if (path.endsWith('.typ')) return 'typst';
	if (path.endsWith('.bib')) return 'bibtex';
	if (path.endsWith('.yml') || path.endsWith('.yaml')) return 'yaml';
	return 'plaintext';
}

function textOf(value: string) {
	return Text.of(value.split(/\r\n?|\n/));
}

class TrackedFile implements WorkspaceFile {
	view: EditorView | null = null;
	stale = false;

	constructor(
		public uri: string,
		public languageId: string,
		public version: number,
		public doc: Text,
		readonly text: Y.Text | null
	) {}

	getView() {
		return this.view;
	}

	current() {
		if (this.view) return this.view.state.doc;
		return this.text ? textOf(this.text.toString()) : this.doc;
	}
}

export class YjsWorkspace extends Workspace {
	files: TrackedFile[] = [];
	private unobserve: (() => void)[] = [];
	private pendingDisplay = new Map<string, (view: EditorView) => void>();

	constructor(
		client: LSPClient,
		private rootUri: string,
		private hooks: WorkspaceHooks
	) {
		super(client);
		for (const { path, text } of hooks.files()) this.track(path, text);
	}

	uriFor(path: string) {
		return `${this.rootUri}/${path.split('/').map(encodeURIComponent).join('/')}`;
	}

	private pathFor(uri: string) {
		const prefix = `${this.rootUri}/`;
		return uri.startsWith(prefix) ? uri.slice(prefix.length).split('/').map(decodeURIComponent).join('/') : null;
	}

	private track(path: string, text: Y.Text) {
		const file = new TrackedFile(this.uriFor(path), languageIdFor(path), 0, textOf(text.toString()), text);
		const observer = () => {
			if (file.view) return;
			file.stale = true;
			this.hooks.scheduleSync();
		};
		text.observe(observer);
		this.unobserve.push(() => text.unobserve(observer));
		this.files.push(file);
	}

	syncFiles() {
		const updates = [];
		for (const file of this.files) {
			const plugin = file.view ? LSPPlugin.get(file.view) : null;
			let changes: ChangeSet;
			if (file.stale) {
				const next = file.current();
				changes = ChangeSet.of({ from: 0, to: file.doc.length, insert: next }, file.doc.length);
			} else if (plugin && !plugin.unsyncedChanges.empty) {
				changes = plugin.unsyncedChanges;
			} else {
				continue;
			}
			const prevDoc = file.doc;
			file.doc = file.current();
			file.version++;
			file.stale = false;
			plugin?.clear();
			updates.push({ file, prevDoc, changes });
		}
		return updates;
	}

	openFile(uri: string, languageId: string, view: EditorView) {
		let file = this.getFile(uri) as TrackedFile | null;
		if (!file) {
			file = new TrackedFile(uri, languageId, 0, view.state.doc, null);
			this.files.push(file);
			file.view = view;
			this.client.didOpen(file);
			return;
		}
		file.view = view;
		if (!file.doc.eq(view.state.doc)) file.stale = true;
		this.pendingDisplay.get(uri)?.(view);
		this.pendingDisplay.delete(uri);
	}

	closeFile(uri: string, view: EditorView) {
		const file = this.getFile(uri) as TrackedFile | null;
		if (!file || file.view !== view) return;
		if (!file.text) {
			this.files = this.files.filter((candidate) => candidate !== file);
			this.client.didClose(uri);
			return;
		}
		if (!file.doc.eq(view.state.doc)) file.stale = true;
		file.view = null;
	}

	displayFile(uri: string): Promise<EditorView | null> {
		const file = this.getFile(uri);
		if (file?.getView()) return Promise.resolve(file.getView());
		const path = this.pathFor(uri);
		if (!file || !path || !this.hooks.openPath) return Promise.resolve(null);
		const opened = new Promise<EditorView | null>((resolve) => {
			this.pendingDisplay.set(uri, resolve);
			setTimeout(() => {
				if (this.pendingDisplay.delete(uri)) resolve(null);
			}, DISPLAY_TIMEOUT_MS);
		});
		this.hooks.openPath(path);
		return opened;
	}

	dispose() {
		for (const stop of this.unobserve) stop();
		this.unobserve = [];
	}
}
