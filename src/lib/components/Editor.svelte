<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorState, Compartment } from '@codemirror/state';
	import { EditorView, lineNumbers, keymap } from '@codemirror/view';
	import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
	import { autocompletion, snippetCompletion, type CompletionContext } from '@codemirror/autocomplete';
	import { typst, TypstParser, typstHighlight } from 'codemirror-lang-typst';
	import { Language } from '@codemirror/language';
	import { yCollab } from 'y-codemirror.next';
	import { text, provider } from '../ts/yjs-setup';
	import { getThemeExtension } from '../ts/themes';
	import { themeStore, darkModeStore, editorViewStore, editorErrors, triggerLspReconnect } from '../ts/store';
	import { page } from '$app/stores';
	import { LSPClient, languageServerExtensions } from "@codemirror/lsp-client";
	import { setDiagnostics, lintGutter } from '@codemirror/lint';

	let editorContainer: HTMLElement;
	let view: EditorView;
	let themeCompartment = new Compartment();
	let lspCompartment = new Compartment();
	let unsubscribeTheme: () => void;
	let unsubscribeDark: () => void;
	let unsubscribeErrors: () => void;
	let unsubscribeLspReconnect: () => void;

	let currentTheme = 'Catppuccin';
	let isDark = true;
	let state: EditorState;
	let client: LSPClient | null = null;
	let lsSocket: WebSocket | null = null;

	const typstOptions = [
		snippetCompletion("let ${name} = ${value}", { label: "let", type: "keyword", info: "Variable declaration" }),
		snippetCompletion("set ${rule}(${value})", { label: "set", type: "keyword", info: "Set rule" }),
		snippetCompletion("show ${selector}: ${rule}", { label: "show", type: "keyword", info: "Show rule" }),
		snippetCompletion("import \"${module}\": ${items}", { label: "import", type: "keyword", info: "Import module" }),
		snippetCompletion("include \"${file}\"", { label: "include", type: "keyword", info: "Include file" }),
		snippetCompletion("if ${condition} {\n\t${}\n}", { label: "if", type: "keyword", info: "If statement" }),
		snippetCompletion("else {\n\t${}\n}", { label: "else", type: "keyword", info: "Else statement" }),
		snippetCompletion("for ${item} in ${collection} {\n\t${}\n}", { label: "for", type: "keyword", info: "For loop" }),
		snippetCompletion("while ${condition} {\n\t${}\n}", { label: "while", type: "keyword", info: "While loop" }),
		snippetCompletion("break", { label: "break", type: "keyword", info: "Break loop" }),
		snippetCompletion("continue", { label: "continue", type: "keyword", info: "Continue loop" }),
		snippetCompletion("return ${value}", { label: "return", type: "keyword", info: "Return value" }),
		snippetCompletion("context", { label: "context", type: "keyword", info: "Context expression" }),
		snippetCompletion("align(${alignment})[${content}]", { label: "align", type: "function", info: "Align content" }),
		snippetCompletion("page(${content})", { label: "page", type: "function", info: "Page configuration" }),
		snippetCompletion("pagebreak()", { label: "pagebreak", type: "function", info: "Break page" }),
		snippetCompletion("colbreak()", { label: "colbreak", type: "function", info: "Break column" }),
		snippetCompletion("place(${alignment})[${content}]", { label: "place", type: "function", info: "Place content" }),
		snippetCompletion("columns(${2})[${content}]", { label: "columns", type: "function", info: "Multiple columns" }),
		snippetCompletion("pad(${10pt})[${content}]", { label: "pad", type: "function", info: "Pad content" }),
		snippetCompletion("stack(dir: ${ttb}, spacing: ${10pt}, ${items})", { label: "stack", type: "function", info: "Stack items" }),
		snippetCompletion("grid(columns: ${2}, gutter: ${10pt}, ${items})", { label: "grid", type: "function", info: "Grid layout" }),
		snippetCompletion("table(columns: ${2}, ${items})", { label: "table", type: "function", info: "Table layout" }),
		snippetCompletion("rect(width: ${100%}, height: ${100%})[${content}]", { label: "rect", type: "function", info: "Draw rectangle" }),
		snippetCompletion("square(size: ${10pt})[${content}]", { label: "square", type: "function", info: "Draw square" }),
		snippetCompletion("circle(radius: ${10pt})[${content}]", { label: "circle", type: "function", info: "Draw circle" }),
		snippetCompletion("ellipse(width: ${20pt}, height: ${10pt})[${content}]", { label: "ellipse", type: "function", info: "Draw ellipse" }),
		snippetCompletion("line(length: ${100%})", { label: "line", type: "function", info: "Draw line" }),
		snippetCompletion("polygon(${vertices})", { label: "polygon", type: "function", info: "Draw polygon" }),
		snippetCompletion("path(${vertices})", { label: "path", type: "function", info: "Draw path" }),
		snippetCompletion("image(\"${path}\", width: ${100%})", { label: "image", type: "function", info: "Insert image" }),
		snippetCompletion("box[${content}]", { label: "box", type: "function", info: "Box inline content" }),
		snippetCompletion("block[${content}]", { label: "block", type: "function", info: "Block content" }),
		snippetCompletion("figure(${content}, caption: [${caption}])", { label: "figure", type: "function", info: "Figure with caption" }),
		snippetCompletion("text(size: ${11pt}, font: \"${Arial}\")[${content}]", { label: "text", type: "function", info: "Text styling" }),
		snippetCompletion("heading(level: ${1})[${title}]", { label: "heading", type: "function", info: "Heading" }),
		snippetCompletion("par[${content}]", { label: "par", type: "function", info: "Paragraph" }),
		snippetCompletion("list([${item}])", { label: "list", type: "function", info: "Bullet list" }),
		snippetCompletion("enum([${item}])", { label: "enum", type: "function", info: "Numbered list" }),
		snippetCompletion("terms([${term}], [${description}])", { label: "terms", type: "function", info: "Terms list" }),
		snippetCompletion("strong[${content}]", { label: "strong", type: "function", info: "Bold text" }),
		snippetCompletion("emph[${content}]", { label: "emph", type: "function", info: "Italic text" }),
		snippetCompletion("underline[${content}]", { label: "underline", type: "function", info: "Underline text" }),
		snippetCompletion("strike[${content}]", { label: "strike", type: "function", info: "Strikethrough text" }),
		snippetCompletion("overline[${content}]", { label: "overline", type: "function", info: "Overline text" }),
		snippetCompletion("sub[${content}]", { label: "sub", type: "function", info: "Subscript text" }),
		snippetCompletion("super[${content}]", { label: "super", type: "function", info: "Superscript text" }),
		snippetCompletion("raw(\"${code}\", block: ${true})", { label: "raw", type: "function", info: "Raw code block" }),
		snippetCompletion("link(\"${url}\")[${text}]", { label: "link", type: "function", info: "Hyperlink" }),
		snippetCompletion("ref(<${label}>)", { label: "ref", type: "function", info: "Reference" }),
		snippetCompletion("cite(<${label}>)", { label: "cite", type: "function", info: "Citation" }),
		snippetCompletion("bibliography(\"${file.bib}\")", { label: "bibliography", type: "function", info: "Bibliography" }),
		snippetCompletion("outline(title: [${Contents}])", { label: "outline", type: "function", info: "Table of contents" }),
		snippetCompletion("rgb(\"${#000000}\")", { label: "rgb", type: "function", info: "RGB Color" }),
		snippetCompletion("cmyk(${0%}, ${0%}, ${0%}, ${100%})", { label: "cmyk", type: "function", info: "CMYK Color" }),
		snippetCompletion("luma(${0%})", { label: "luma", type: "function", info: "Luma (Grayscale) Color" }),
		snippetCompletion("color", { label: "color", type: "variable" }),
		snippetCompletion("gradient", { label: "gradient", type: "variable" }),
		snippetCompletion("pattern(size: (${10pt}, ${10pt}))[${content}]", { label: "pattern", type: "function", info: "Fill pattern" }),
		snippetCompletion("type(${value})", { label: "type", type: "function", info: "Get type of value" }),
		snippetCompletion("repr(${value})", { label: "repr", type: "function", info: "String representation" }),
		snippetCompletion("str(${value})", { label: "str", type: "function", info: "Convert to string" }),
		snippetCompletion("int(${value})", { label: "int", type: "function", info: "Convert to integer" }),
		snippetCompletion("float(${value})", { label: "float", type: "function", info: "Convert to float" }),
		snippetCompletion("datetime(year: ${2024}, month: ${1}, day: ${1})", { label: "datetime", type: "function", info: "Date and time" }),
		snippetCompletion("math", { label: "math", type: "variable", info: "Math module" }),
		snippetCompletion("calc", { label: "calc", type: "variable", info: "Calc module" }),
		snippetCompletion("sys", { label: "sys", type: "variable", info: "System module" }),
		snippetCompletion("frac(${num}, ${denom})", { label: "frac", type: "function", info: "Fraction (Math)" }),
		snippetCompletion("binom(${n}, ${k})", { label: "binom", type: "function", info: "Binomial (Math)" }),
		snippetCompletion("mat(${1}, ${2}; ${3}, ${4})", { label: "mat", type: "function", info: "Matrix (Math)" }),
		snippetCompletion("vec(${1}, ${2})", { label: "vec", type: "function", info: "Vector (Math)" }),
		snippetCompletion("cases(${a}, ${b})", { label: "cases", type: "function", info: "Cases (Math)" }),
		snippetCompletion("sqrt(${x})", { label: "sqrt", type: "function", info: "Square root (Math)" }),
		snippetCompletion("root(${3}, ${x})", { label: "root", type: "function", info: "N-th root (Math)" }),
		snippetCompletion("abs(${x})", { label: "abs", type: "function", info: "Absolute value (Math)" }),
		snippetCompletion("norm(${x})", { label: "norm", type: "function", info: "Norm (Math)" }),
		snippetCompletion("floor(${x})", { label: "floor", type: "function", info: "Floor (Math)" }),
		snippetCompletion("ceil(${x})", { label: "ceil", type: "function", info: "Ceiling (Math)" }),
		snippetCompletion("round(${x})", { label: "round", type: "function", info: "Round (Math)" }),
		snippetCompletion("cancel(${x})", { label: "cancel", type: "function", info: "Cancel/strike (Math)" }),
		snippetCompletion("attach(${base}, t: ${top}, b: ${bottom})", { label: "attach", type: "function", info: "Attach scripts (Math)" }),
		snippetCompletion("scripts(${expr})", { label: "scripts", type: "function", info: "Scripts (Math)" }),
		snippetCompletion("limits(${expr})", { label: "limits", type: "function", info: "Limits (Math)" }),
		snippetCompletion("op(\"${name}\")", { label: "op", type: "function", info: "Operator (Math)" }),
		snippetCompletion("lr(${expr})", { label: "lr", type: "function", info: "Left/Right scales (Math)" }),
		snippetCompletion("mid(${|})", { label: "mid", type: "function", info: "Mid delimiter (Math)" })
	];

	function typstCompletions(context: CompletionContext) {
		let word = context.matchBefore(/[\w#]*/);
		if (!word || (word.from == word.to && !context.explicit)) return null;
		
		let textBefore = word.text;
		if (textBefore.startsWith('#')) {
			textBefore = textBefore.substring(1); 
		}

		return {
			from: word.text.startsWith('#') ? word.from + 1 : word.from,
			options: typstOptions,
			validFor: /^[\w]*$/
		};
	}

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
				lintGutter(),
				history(),
				keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab] as any),
				myLang,
				yCollab(text, provider.awareness),
				autocompletion({ override: [typstCompletions] }),
				themeCompartment.of(getThemeExtension(currentTheme as any, isDark)),
				lspCompartment.of([]),
				EditorView.lineWrapping,
				EditorView.theme({
					'&': { height: '100%', fontSize: '14px' },
					'.cm-scroller': { overflow: 'auto' },
					'.cm-tooltip': { maxWidth: '500px' },
					'.cm-tooltip-hover': { maxHeight: '300px', overflow: 'auto' }
				}),
			],
		});

		view = new EditorView({
			state,
			parent: editorContainer,
		});
		
		editorViewStore.set(view);

		unsubscribeErrors = editorErrors.subscribe((errors) => {
			if (view) {
				const docLen = view.state.doc.length;
				const safeDiagnostics = errors.filter(e => e.from != null && e.to != null).map(e => {
					let from = e.from as number;
					let to = e.to as number;
					if (from < 0) from = 0;
					if (to > docLen) to = docLen;
					if (from > to) from = to;
					return {
						from,
						to,
						severity: (e.severity.toLowerCase().includes('warning') ? 'warning' : 'error') as 'warning' | 'error',
						message: e.message
					};
				});
				view.dispatch(setDiagnostics(view.state, safeDiagnostics));
			}
		});

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

		const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		const host = window.location.host;
		const docId = $page.params.id;

		let lsHandlers: ((value: string) => void)[] = [];
		let lspInitialized = false;

		const transport = {
			send(message: string) { if (lsSocket?.readyState === WebSocket.OPEN) lsSocket.send(message); },
			subscribe(handler: (value: string) => void) { lsHandlers.push(handler); },
			unsubscribe(handler: (value: string) => void) { lsHandlers = lsHandlers.filter(h => h != handler); }
		};

		function connectLsp() {
			if (lsSocket) {
				lsSocket.close();
				lsSocket = null;
			}
			
			lspInitialized = false;
			lsHandlers = [];
			
			lsSocket = new WebSocket(`${protocol}//${host}/api/lsp/${docId}`);

			lsSocket.onmessage = e => {
				const data = e.data.toString();
				if (!lspInitialized) {
					try {
						const msg = JSON.parse(data);
						if (msg.type === 'init') {
							lspInitialized = true;
							
							// Recreate the client because the backend started a completely new LSP process
							// which requires a fresh 'initialize' handshake.
							client = new LSPClient({
								rootUri: msg.rootUri,
								timeout: 10000,
								extensions: languageServerExtensions()
							}).connect(transport);

							view.dispatch({
								effects: lspCompartment.reconfigure(client.plugin(`${msg.rootUri}/${docId}.typ`, 'typst'))
							});
							return;
						}
					} catch (err) {
						// Fallthrough
					}
				}
				
				let processedData = data;
				if (lspInitialized) {
					try {
						const msg = JSON.parse(data);
						if (msg.method === 'textDocument/publishDiagnostics' && msg.params && msg.params.diagnostics) {
							msg.params.diagnostics = msg.params.diagnostics.filter((d: any) => !d.message.toLowerCase().includes('unknown font family'));
							processedData = JSON.stringify(msg);
						}
					} catch (err) {}
				}
				
				for (let h of lsHandlers) h(processedData);
			};

			lsSocket.onopen = () => {
				// Waiting for init message from server
			};
		}
		
		connectLsp();
		
		unsubscribeLspReconnect = triggerLspReconnect.subscribe((val) => {
			if (val > 0) {
				connectLsp();
			}
		});
	});

	onDestroy(() => {
		if (lsSocket) lsSocket.close();
		if (unsubscribeTheme) unsubscribeTheme();
		if (unsubscribeDark) unsubscribeDark();
		if (unsubscribeErrors) unsubscribeErrors();
		if (unsubscribeLspReconnect) unsubscribeLspReconnect();
		if (view) {
			view.destroy();
		}
		editorViewStore.set(null);
	});
</script>

<div class="h-full w-full overflow-hidden focus-within:ring-2 focus-within:ring-inset focus-within:ring-blue-500/20" bind:this={editorContainer}></div>
