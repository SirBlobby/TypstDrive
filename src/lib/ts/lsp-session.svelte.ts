import { LSPClient, type Transport } from '@codemirror/lsp-client';
import type { Extension } from '@codemirror/state';
import { writable } from 'svelte/store';
import { typstLspExtensions } from './editor-lsp';
import { YjsWorkspace, languageIdFor, type TrackedText } from './lsp-workspace';

type Diagnostic = { message: string; range?: { start?: { line?: number } } };

export type LspSessionOptions = {
	path: string;
	files: () => Iterable<TrackedText>;
	openPath?: (path: string) => void;
};

const REQUEST_TIMEOUT_MS = 10000;
const BACKGROUND_SYNC_MS = 500;
const RETRY_MIN_MS = 2000;
const RETRY_MAX_MS = 60000;
const HIDDEN_IDLE_MS = 10 * 60 * 1000;

function asLocation(item: unknown) {
	if (!item || typeof item !== 'object' || !('targetUri' in item)) return item;
	const link = item as { targetUri: string; targetSelectionRange: unknown };
	return { uri: link.targetUri, range: link.targetSelectionRange };
}

export const activeLspSession = writable<LspSession | null>(null);

export class LspSession {
	client = $state.raw<LSPClient | null>(null);
	skipDiagnostic: ((diagnostic: Diagnostic) => boolean) | null = null;

	private workspace: YjsWorkspace | null = null;
	private socket: WebSocket | null = null;
	private handlers: ((message: string) => void)[] = [];
	private lastDiagnostics = new Map<string, string>();
	private syncTimer: ReturnType<typeof setTimeout> | undefined;
	private retryTimer: ReturnType<typeof setTimeout> | undefined;
	private idleTimer: ReturnType<typeof setTimeout> | undefined;
	private retryDelay = RETRY_MIN_MS;
	private closed = false;

	constructor(private options: LspSessionOptions) {
		document.addEventListener('visibilitychange', this.onVisibilityChange);
		this.connect();
	}

	plugin(path: string): Extension {
		if (!this.client || !this.workspace) return [];
		return this.client.plugin(this.workspace.uriFor(path), languageIdFor(path));
	}

	restart() {
		if (this.closed) return;
		this.retryDelay = RETRY_MIN_MS;
		this.connect();
	}

	close() {
		this.closed = true;
		document.removeEventListener('visibilitychange', this.onVisibilityChange);
		clearTimeout(this.idleTimer);
		this.disconnect();
	}

	private transport: Transport = {
		send: (message) => {
			if (this.socket?.readyState === WebSocket.OPEN) this.socket.send(message);
		},
		subscribe: (handler) => {
			this.handlers.push(handler);
		},
		unsubscribe: (handler) => {
			this.handlers = this.handlers.filter((candidate) => candidate !== handler);
		}
	};

	private connect() {
		this.disconnect();
		const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		const socket = new WebSocket(`${protocol}//${window.location.host}${this.options.path}`);
		this.socket = socket;
		socket.onmessage = (event) => this.receive(String(event.data));
		socket.onclose = () => {
			if (this.socket !== socket) return;
			this.disconnect();
			if (!this.closed && document.visibilityState === 'visible') this.scheduleRetry();
		};
	}

	private disconnect() {
		clearTimeout(this.retryTimer);
		clearTimeout(this.syncTimer);
		const socket = this.socket;
		this.socket = null;
		socket?.close();
		this.client?.disconnect();
		this.workspace?.dispose();
		this.workspace = null;
		this.handlers = [];
		this.lastDiagnostics.clear();
		this.client = null;
	}

	private scheduleRetry() {
		this.retryTimer = setTimeout(() => this.connect(), this.retryDelay);
		this.retryDelay = Math.min(this.retryDelay * 2, RETRY_MAX_MS);
	}

	private receive(data: string) {
		let message;
		try {
			message = JSON.parse(data);
		} catch {
			return;
		}
		if (!this.client) {
			if (message.type === 'init') this.start(message.rootUri);
			return;
		}
		if (message.method === 'textDocument/publishDiagnostics' && !this.keepDiagnostics(message.params)) return;
		if (Array.isArray(message.result)) message.result = message.result.map(asLocation);
		const forwarded = JSON.stringify(message);
		for (const handler of this.handlers) handler(forwarded);
	}

	private start(rootUri: string) {
		this.retryDelay = RETRY_MIN_MS;
		const client = new LSPClient({
			rootUri,
			timeout: REQUEST_TIMEOUT_MS,
			extensions: typstLspExtensions(),
			workspace: (client) => {
				this.workspace = new YjsWorkspace(client, rootUri, {
					files: this.options.files,
					openPath: this.options.openPath,
					scheduleSync: () => this.scheduleSync()
				});
				return this.workspace;
			}
		});
		client.connect(this.transport);
		this.client = client;
	}

	private scheduleSync() {
		clearTimeout(this.syncTimer);
		this.syncTimer = setTimeout(() => this.client?.sync(), BACKGROUND_SYNC_MS);
	}

	private keepDiagnostics(params: { uri: string; version?: number; diagnostics: Diagnostic[] }) {
		params.diagnostics = params.diagnostics.filter(
			(diagnostic) => !diagnostic.message.toLowerCase().includes('unknown font family') && !this.skipDiagnostic?.(diagnostic)
		);
		const snapshot = `${params.version ?? ''}:${JSON.stringify(params.diagnostics)}`;
		if (this.lastDiagnostics.get(params.uri) === snapshot) return false;
		this.lastDiagnostics.set(params.uri, snapshot);
		return true;
	}

	private onVisibilityChange = () => {
		clearTimeout(this.idleTimer);
		if (document.visibilityState === 'hidden') {
			this.idleTimer = setTimeout(() => this.disconnect(), HIDDEN_IDLE_MS);
		} else if (!this.socket && !this.closed) {
			this.restart();
		}
	};
}
