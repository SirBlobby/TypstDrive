type SavedTabs = {
	tabs: string[];
	active: string;
};

function storageKey(projectId: string) {
	return `project-tabs:${projectId}`;
}

export function loadTabs(projectId: string): SavedTabs | null {
	try {
		const saved = JSON.parse(localStorage.getItem(storageKey(projectId)) ?? 'null');
		if (!saved || !Array.isArray(saved.tabs) || typeof saved.active !== 'string') return null;
		return { tabs: saved.tabs.filter((tab: unknown): tab is string => typeof tab === 'string'), active: saved.active };
	} catch {
		return null;
	}
}

export function saveTabs(projectId: string, tabs: string[], active: string) {
	try {
		localStorage.setItem(storageKey(projectId), JSON.stringify({ tabs, active }));
	} catch {
		return;
	}
}

export function tabAfterClosing(tabs: string[], closedId: string): string {
	const position = tabs.indexOf(closedId);
	const remaining = tabs.filter((tab) => tab !== closedId);
	return remaining[Math.min(position, remaining.length - 1)] ?? '';
}
