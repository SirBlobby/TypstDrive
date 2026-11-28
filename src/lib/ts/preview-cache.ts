import type { PreviewPages } from './typst-api';


const DB_NAME = 'typstdrive-preview-cache';
const DB_VERSION = 1;
const PAGES = 'pages';
const PROJECTS = 'projects';
const META = 'meta';
const MAX_PROJECTS = 6;
const MAX_CHARS = 200_000_000;

interface ProjectRecord {
	key: string;
	hashes: string[];
	chars: number;
	usedAt: number;
}

function request<T>(req: IDBRequest<T>): Promise<T> {
	return new Promise((resolve, reject) => {
		req.onsuccess = () => resolve(req.result);
		req.onerror = () => reject(req.error);
	});
}

function completion(tx: IDBTransaction): Promise<void> {
	return new Promise((resolve, reject) => {
		tx.oncomplete = () => resolve();
		tx.onerror = () => reject(tx.error);
		tx.onabort = () => reject(tx.error);
	});
}

let database: Promise<IDBDatabase | null> | null = null;

function openDatabase(): Promise<IDBDatabase | null> {
	if (database) return database;
	database = new Promise((resolve) => {
		try {
			const req = indexedDB.open(DB_NAME, DB_VERSION);
			req.onupgradeneeded = () => {
				const db = req.result;
				db.createObjectStore(PAGES);
				db.createObjectStore(PROJECTS, { keyPath: 'key' });
				db.createObjectStore(META);
			};
			req.onsuccess = () => {
				const db = req.result;
				db.onversionchange = () => {
					db.close();
					database = null;
				};
				resolve(db);
			};
			req.onerror = () => resolve(null);
			req.onblocked = () => resolve(null);
		} catch {
			resolve(null);
		}
	});
	return database;
}

function projectKey(userId: string, projectId: string) {
	return `${userId}:${projectId}`;
}

async function pageHash(svg: string): Promise<string> {
	const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(svg)));
	let hex = '';
	for (let i = 0; i < 16; i++) hex += digest[i].toString(16).padStart(2, '0');
	return hex;
}

export async function retainPreviewCacheFor(userId: string | null): Promise<void> {
	try {
		const db = await openDatabase();
		if (!db) return;
		const tx = db.transaction([META, PAGES, PROJECTS], 'readwrite');
		const meta = tx.objectStore(META);
		const owner = await request(meta.get('owner'));
		if (owner !== userId) {
			tx.objectStore(PAGES).clear();
			tx.objectStore(PROJECTS).clear();
			if (userId === null) meta.delete('owner');
			else meta.put(userId, 'owner');
		}
		await completion(tx);
	} catch {
	}
}

export async function loadCachedPreview(userId: string, projectId: string): Promise<PreviewPages | null> {
	try {
		const db = await openDatabase();
		if (!db) return null;
		const tx = db.transaction([PROJECTS, PAGES], 'readonly');
		const record = (await request(tx.objectStore(PROJECTS).get(projectKey(userId, projectId)))) as ProjectRecord | undefined;
		if (!record || !Array.isArray(record.hashes) || record.hashes.length === 0) return null;
		const store = tx.objectStore(PAGES);
		const svgs = await Promise.all(record.hashes.map((hash) => request(store.get(hash)) as Promise<unknown>));
		if (!svgs.every((svg): svg is string => typeof svg === 'string')) return null;
		const hashes = await Promise.all(svgs.map(pageHash));
		if (hashes.some((hash, i) => hash !== record.hashes[i])) return null;
		touch(db, record);
		return { svgs, hashes };
	} catch {
		return null;
	}
}

function touch(db: IDBDatabase, record: ProjectRecord) {
	try {
		db.transaction(PROJECTS, 'readwrite').objectStore(PROJECTS).put({ ...record, usedAt: Date.now() });
	} catch {
	}
}

export async function saveCachedPreview(userId: string, projectId: string, pages: PreviewPages): Promise<void> {
	const { svgs, hashes } = pages;
	if (hashes.length === 0 || hashes.length !== svgs.length) return;
	const chars = svgs.reduce((total, svg) => total + svg.length, 0);
	if (chars > MAX_CHARS) return;
	try {
		const db = await openDatabase();
		if (!db) return;
		const tx = db.transaction([PROJECTS, PAGES], 'readwrite');
		const projects = tx.objectStore(PROJECTS);
		const store = tx.objectStore(PAGES);
		const key = projectKey(userId, projectId);
		const stored = new Set((await request(store.getAllKeys())) as string[]);
		hashes.forEach((hash, i) => {
			if (!stored.has(hash)) store.put(svgs[i], hash);
		});
		const record: ProjectRecord = { key, hashes: [...hashes], chars, usedAt: Date.now() };
		projects.put(record);

		const others = ((await request(projects.getAll())) as ProjectRecord[]).filter((p) => p.key !== key).sort((a, b) => b.usedAt - a.usedAt);
		const used = new Set(record.hashes);
		let total = chars;
		let kept = 1;
		for (const project of others) {
			if (kept < MAX_PROJECTS && total + project.chars <= MAX_CHARS) {
				total += project.chars;
				kept++;
				for (const hash of project.hashes) used.add(hash);
			} else {
				projects.delete(project.key);
			}
		}
		for (const hash of stored) if (!used.has(hash)) store.delete(hash);
		await completion(tx);
	} catch {
	}
}
