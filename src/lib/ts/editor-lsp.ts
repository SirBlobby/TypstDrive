import {
	serverCompletion,
	serverDiagnostics,
	signatureHelp,
	formatKeymap,
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
		keymap.of([...formatKeymap, ...renameKeymap, ...jumpToDefinitionKeymap, ...findReferencesKeymap])
	];
}
