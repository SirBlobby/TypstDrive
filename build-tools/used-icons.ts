import { readFileSync, readdirSync, statSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import type { Plugin } from 'vite';

const MODULE_ID = 'virtual:used-icons';
const RESOLVED_ID = '\0' + MODULE_ID;
const ICON_SETS = ['mdi', 'ph', 'simple-icons'];
const ICON_NAME = new RegExp(`\\b(${ICON_SETS.join('|')}):([a-z0-9]+(?:-[a-z0-9]+)*)`, 'g');
const SOURCE_EXTENSIONS = ['.svelte', '.ts'];

type IconSet = {
	prefix: string;
	icons: Record<string, unknown>;
	aliases?: Record<string, { parent: string }>;
	width?: number;
	height?: number;
};

function sourceFiles(folder: string): string[] {
	return readdirSync(folder).flatMap((entry) => {
		const path = join(folder, entry);
		if (statSync(path).isDirectory()) return sourceFiles(path);
		return SOURCE_EXTENSIONS.some((extension) => path.endsWith(extension)) ? [path] : [];
	});
}

function usedNames(folder: string): Map<string, Set<string>> {
	const used = new Map<string, Set<string>>(ICON_SETS.map((set) => [set, new Set()]));
	for (const file of sourceFiles(folder)) {
		for (const [, set, name] of readFileSync(file, 'utf8').matchAll(ICON_NAME)) used.get(set)?.add(name);
	}
	return used;
}

function subset(iconSet: IconSet, names: Set<string>): IconSet {
	const icons: Record<string, unknown> = {};
	const aliases: Record<string, { parent: string }> = {};
	for (const name of names) {
		let current = name;
		while (iconSet.aliases?.[current] && !aliases[current]) {
			aliases[current] = iconSet.aliases[current];
			current = iconSet.aliases[current].parent;
		}
		if (iconSet.icons[current]) icons[current] = iconSet.icons[current];
	}
	return { prefix: iconSet.prefix, icons, aliases, width: iconSet.width, height: iconSet.height };
}

export function usedIcons(sourceFolder: string): Plugin {
	const require = createRequire(import.meta.url);
	return {
		name: 'used-icons',
		resolveId(id) {
			return id === MODULE_ID ? RESOLVED_ID : null;
		},
		load(id) {
			if (id !== RESOLVED_ID) return null;
			const used = usedNames(sourceFolder);
			const collections = ICON_SETS.map((set) => {
				const iconSet: IconSet = require(`@iconify-json/${set}/icons.json`);
				return subset(iconSet, used.get(set) ?? new Set());
			});
			return `export default ${JSON.stringify(collections)};`;
		},
		handleHotUpdate({ file, server }) {
			if (!file.startsWith(sourceFolder)) return;
			const module = server.moduleGraph.getModuleById(RESOLVED_ID);
			if (module) server.moduleGraph.invalidateModule(module);
		}
	};
}
