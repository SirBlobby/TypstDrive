import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), wasm()],
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
