import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { relative, sep } from 'node:path';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

function useRunesOutsideLibraries({ filename }: { filename: string }) {
	const relativePath = relative(import.meta.dirname, filename);
	const pathSegments = relativePath.toLowerCase().split(sep);
	const isExternalLibrary = pathSegments.includes('node_modules');
	return isExternalLibrary ? undefined : true;
}

const svelteKitOptions = {
	alias: {
		$lib: 'src/lib'
	},
	compilerOptions: {
		runes: useRunesOutsideLibraries
	},
	adapter: adapter({
		fallback: 'index.html'
	}),
	prerender: {
		entries: ['*' as const],
		handleUnseenRoutes: 'ignore' as const
	}
};

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(svelteKitOptions), wasm()],
	server: {
		proxy: {
			'/api': 'http://127.0.0.1:3000',
			'/yjs': {
				target: 'ws://127.0.0.1:3000',
				ws: true,
			},
		},
	},
	optimizeDeps: {
		exclude: ['codemirror-lang-typst']
	},
	build: {
		target: 'esnext'
	}
});
