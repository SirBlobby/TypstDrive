import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import type { Config } from '@sveltejs/kit';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

function useRunesOutsideLibraries({ filename }: { filename: string }) {
	const pathSegments = filename.toLowerCase().split(/[\\/]/);
	const isExternalLibrary = pathSegments.includes('node_modules');
	return isExternalLibrary ? undefined : true;
}

const contentSecurityPolicy: Config['csp'] = {
	mode: 'hash',
	directives: {
		'default-src': ['self'],
		'script-src': ['self', 'wasm-unsafe-eval'],
		'style-src': ['self', 'unsafe-inline', 'https://fonts.googleapis.com'],
		'font-src': ['self', 'data:', 'https://fonts.gstatic.com'],
		'img-src': ['self', 'data:', 'blob:', 'https:'],
		'connect-src': ['self', 'https://api.github.com', 'https://api.iconify.design', 'https://api.simplesvg.com', 'https://api.unisvg.com'],
		'frame-src': ['self', 'blob:'],
		'worker-src': ['self', 'blob:'],
		'object-src': ['none'],
		'base-uri': ['self'],
		'form-action': ['self']
	}
};

const svelteKitOptions = {
	compilerOptions: {
		runes: useRunesOutsideLibraries
	},
	csp: contentSecurityPolicy,
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
			'/pdf': 'http://127.0.0.1:3000',
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
