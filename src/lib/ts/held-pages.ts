import { assemblePages, type CompileResponse, type PreviewPages } from './typst-api';

export class HeldPages {
	private hashes: string[] = [];
	private svgs: string[] = [];

	knownHashes(): string[] {
		return [...new Set(this.hashes)];
	}

	assemble(response: CompileResponse): PreviewPages | undefined {
		const held = new Map(this.hashes.map((hash, index) => [hash, this.svgs[index]]));
		const pages = assemblePages(response, held);
		if (pages) {
			this.hashes = pages.hashes;
			this.svgs = pages.svgs;
		} else {
			this.clear();
		}
		return pages;
	}

	clear() {
		this.hashes = [];
		this.svgs = [];
	}
}
