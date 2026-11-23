import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

function useRunesOutsideLibraries({ filename }: { filename: string }) {
	const pathSegments = filename.toLowerCase().split(/[\\/]/);
	const isExternalLibrary = pathSegments.includes('node_modules');
	return isExternalLibrary ? undefined : true;
}

const iconApiHosts = ['https://api.iconify.design', 'https://api.simplesvg.com', 'https://api.unisvg.com'];

const contentSecurityPolicy = {
	mode: 'hash' as const,
	directives: {
		'default-src': ['self' as const],
		'script-src': ['self' as const, 'wasm-unsafe-eval' as const],
		'style-src': ['self' as const, 'unsafe-inline' as const, 'https://fonts.googleapis.com'],
		'font-src': ['self' as const, 'data:' as const, 'https://fonts.gstatic.com'],
		'img-src': ['self' as const, 'data:' as const, 'blob:' as const, 'https:' as const],
		'connect-src': ['self' as const, 'https://api.github.com', ...iconApiHosts],
		'frame-src': ['self' as const, 'blob:' as const],
		'worker-src': ['self' as const, 'blob:' as const],
		'object-src': ['none' as const],
		'base-uri': ['self' as const],
		'form-action': ['self' as const]
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
