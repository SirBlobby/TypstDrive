import { StreamLanguage, type StringStream, type StreamParser } from '@codemirror/language';

interface BibtexState {
	depth: number;
	expectKey: boolean;
	inQuote: boolean;
}

function valueToken(stream: StringStream, state: BibtexState) {
	if (stream.match(/^[^"{}\\]+/)) return 'string';
	const character = stream.next();
	if (character === '\\') stream.next();
	else if (character === '{') state.depth++;
	else if (character === '}') state.depth = Math.max(1, state.depth - 1);
	else if (character === '"' && state.inQuote && state.depth === 1) state.inQuote = false;
	return 'string';
}

function outsideEntryToken(stream: StringStream, state: BibtexState) {
	if (stream.eat('%')) {
		stream.skipToEnd();
		return 'comment';
	}
	if (stream.match(/^@[A-Za-z]+/)) return 'keyword';
	if (stream.eat('{') || stream.eat('(')) {
		state.depth = 1;
		state.expectKey = true;
		return 'bracket';
	}
	if (!stream.match(/^[^@%{(]+/)) stream.next();
	return 'comment';
}

function entryToken(stream: StringStream, state: BibtexState) {
	if (state.expectKey) {
		state.expectKey = false;
		if (stream.match(/^[^,\s})]+/)) return 'labelName';
	}
	if (stream.eat('}') || stream.eat(')')) {
		state.depth = 0;
		return 'bracket';
	}
	if (stream.eat('{')) {
		state.depth = 2;
		return 'string';
	}
	if (stream.eat('"')) {
		state.inQuote = true;
		return 'string';
	}
	if (stream.eat(',')) return 'punctuation';
	if (stream.eat('=') || stream.eat('#')) return 'operator';
	if (stream.match(/^\d+/)) return 'number';
	if (stream.match(/^[A-Za-z][\w:.+-]*(?=\s*=)/)) return 'propertyName';
	if (stream.match(/^[A-Za-z][\w:.+-]*/)) return 'variableName';
	stream.next();
	return null;
}

const bibtexParser: StreamParser<BibtexState> = {
	name: 'bibtex',
	startState: () => ({ depth: 0, expectKey: false, inQuote: false }),
	copyState: (state) => ({ ...state }),
	token(stream, state) {
		if (stream.eatSpace()) return null;
		if (state.inQuote || state.depth > 1) return valueToken(stream, state);
		if (state.depth === 0) return outsideEntryToken(stream, state);
		return entryToken(stream, state);
	},
	languageData: {
		commentTokens: { line: '%' }
	}
};

export function bibtex() {
	return StreamLanguage.define(bibtexParser);
}
