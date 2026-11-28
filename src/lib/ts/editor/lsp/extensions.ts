import {
	serverCompletion,
	serverDiagnostics,
	signatureHelp,
	renameKeymap,
	jumpToDefinitionKeymap,
	findReferencesKeymap
} from '@codemirror/lsp-client';
import { keymap } from '@codemirror/view';

export function typstLspExtensions() {
	return [
		serverCompletion(),
		signatureHelp(),
		serverDiagnostics(),
		keymap.of([...renameKeymap, ...jumpToDefinitionKeymap, ...findReferencesKeymap])
	];
}
