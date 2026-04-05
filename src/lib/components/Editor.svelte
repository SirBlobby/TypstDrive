<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorState, Compartment } from '@codemirror/state';
	import { EditorView, lineNumbers, keymap } from '@codemirror/view';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { typst, TypstParser, typstHighlight } from 'codemirror-lang-typst';
	import { Language } from '@codemirror/language';
	import { yCollab } from 'y-codemirror.next';
	import { text, provider } from '../ts/yjs-setup';
	import { getThemeExtension } from '../ts/themes';
	import { themeStore, darkModeStore, editorViewStore } from '../ts/store';

	let editorContainer: HTMLElement;
	let view: EditorView;
	let themeCompartment = new Compartment();
	let unsubscribeTheme: () => void;
	let unsubscribeDark: () => void;

	let currentTheme = 'Catppuccin';
	let isDark = true;
	let state: EditorState;

	onMount(() => {
		if (!text || !provider) return;

		themeStore.subscribe(t => { currentTheme = t; })();
		darkModeStore.subscribe(d => { isDark = d; })();

		const t = typst();
		const myParser = new (TypstParser as any)(typstHighlight);
		const myLang = new Language(
			t.language.data,
			myParser,
			[myParser.updateListener()],
			'typst'
		);

		state = EditorState.create({
			doc: text.toString(),
			extensions: [
				lineNumbers(),
				history(),
				keymap.of([...defaultKeymap, ...historyKeymap] as any),
				myLang,
				yCollab(text, provider.awareness),
				themeCompartment.of(getThemeExtension(currentTheme as any, isDark)),
				EditorView.lineWrapping,
				EditorView.theme({
					'&': { height: '100%', fontSize: '14px' },
					'.cm-scroller': { overflow: 'auto' },
				}),
			],
		});

		view = new EditorView({
			state,
			parent: editorContainer,
		});
		
		editorViewStore.set(view);

		unsubscribeTheme = themeStore.subscribe((themeName) => {
			if (view) {
				view.dispatch({
					effects: themeCompartment.reconfigure(getThemeExtension(themeName as any, isDark))
				});
				currentTheme = themeName;
			}
		});

		unsubscribeDark = darkModeStore.subscribe((dark) => {
			if (view) {
				view.dispatch({
					effects: themeCompartment.reconfigure(getThemeExtension(currentTheme as any, dark))
				});
				isDark = dark;
			}
		});
	});

	onDestroy(() => {
		if (unsubscribeTheme) unsubscribeTheme();
		if (unsubscribeDark) unsubscribeDark();
		if (view) {
			view.destroy();
		}
		editorViewStore.set(null);
	});
</script>

<div class="h-full w-full overflow-hidden focus-within:ring-2 focus-within:ring-inset focus-within:ring-blue-500/20" bind:this={editorContainer}></div>
