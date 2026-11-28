import type * as Y from 'yjs';

export function insertTypstSetting(text: Y.Text, setting: string, value: string) {
	const content = text.toString();
	const rulePattern = new RegExp(`^#set\\s+${setting}\\s*\\(([^)]*)\\)`, 'm');
	const match = content.match(rulePattern);

	const [propertyKey, ...propertyValueParts] = value.split(':');
	const propertyName = propertyKey.trim();
	const propertyValue = propertyValueParts.join(':').trim();

	if (!match) {
		text.insert(0, `#set ${setting}(${value})\n`);
		return;
	}

	const index = match.index!;
	const oldArguments = match[1];
	const propertyPattern = new RegExp(`${propertyName}\\s*:\\s*(?:\\([^)]*\\)|"[^"]*"|[^,)]+)`);
	let newArguments;
	if (propertyPattern.test(oldArguments)) {
		newArguments = oldArguments.replace(propertyPattern, `${propertyName}: ${propertyValue}`);
	} else {
		newArguments = oldArguments.trim() ? `${oldArguments}, ${propertyName}: ${propertyValue}` : `${propertyName}: ${propertyValue}`;
	}

	text.delete(index, match[0].length);
	text.insert(index, `#set ${setting}(${newArguments})`);
}
