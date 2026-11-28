export type DiffLineKind = 'same' | 'added' | 'removed';

export interface DiffLine {
    kind: DiffLineKind;
    text: string;
    oldNumber: number | null;
    newNumber: number | null;
}

export interface DiffSummary {
    added: number;
    removed: number;
}

const MAX_TABLE_CELLS = 4_000_000;

function commonEdges(oldLines: string[], newLines: string[]) {
    let start = 0;
    while (start < oldLines.length && start < newLines.length && oldLines[start] === newLines[start]) start++;
    let endOffset = 0;
    while (
        endOffset < oldLines.length - start &&
        endOffset < newLines.length - start &&
        oldLines[oldLines.length - 1 - endOffset] === newLines[newLines.length - 1 - endOffset]
    ) {
        endOffset++;
    }
    return { start, endOffset };
}

function replaceAll(oldMiddle: string[], newMiddle: string[], offset: number): DiffLine[] {
    return [
        ...oldMiddle.map((text, index) => ({ kind: 'removed' as const, text, oldNumber: offset + index + 1, newNumber: null })),
        ...newMiddle.map((text, index) => ({ kind: 'added' as const, text, oldNumber: null, newNumber: offset + index + 1 }))
    ];
}

function diffMiddle(oldMiddle: string[], newMiddle: string[], offset: number): DiffLine[] {
    const rowCount = oldMiddle.length;
    const columnCount = newMiddle.length;
    if (rowCount === 0 || columnCount === 0 || rowCount * columnCount > MAX_TABLE_CELLS) {
        return replaceAll(oldMiddle, newMiddle, offset);
    }

    const common = Array.from({ length: rowCount + 1 }, () => new Uint32Array(columnCount + 1));
    for (let row = rowCount - 1; row >= 0; row--) {
        for (let column = columnCount - 1; column >= 0; column--) {
            common[row][column] = oldMiddle[row] === newMiddle[column]
                ? common[row + 1][column + 1] + 1
                : Math.max(common[row + 1][column], common[row][column + 1]);
        }
    }

    const lines: DiffLine[] = [];
    let row = 0;
    let column = 0;
    while (row < rowCount && column < columnCount) {
        if (oldMiddle[row] === newMiddle[column]) {
            lines.push({ kind: 'same', text: oldMiddle[row], oldNumber: offset + row + 1, newNumber: offset + column + 1 });
            row++;
            column++;
        } else if (common[row + 1][column] >= common[row][column + 1]) {
            lines.push({ kind: 'removed', text: oldMiddle[row], oldNumber: offset + row + 1, newNumber: null });
            row++;
        } else {
            lines.push({ kind: 'added', text: newMiddle[column], oldNumber: null, newNumber: offset + column + 1 });
            column++;
        }
    }
    for (; row < rowCount; row++) {
        lines.push({ kind: 'removed', text: oldMiddle[row], oldNumber: offset + row + 1, newNumber: null });
    }
    for (; column < columnCount; column++) {
        lines.push({ kind: 'added', text: newMiddle[column], oldNumber: null, newNumber: offset + column + 1 });
    }
    return lines;
}

export function diffLines(oldText: string, newText: string): DiffLine[] {
    const oldLines = oldText.split('\n');
    const newLines = newText.split('\n');
    const { start, endOffset } = commonEdges(oldLines, newLines);
    const lines: DiffLine[] = [];

    for (let index = 0; index < start; index++) {
        lines.push({ kind: 'same', text: oldLines[index], oldNumber: index + 1, newNumber: index + 1 });
    }

    const oldMiddle = oldLines.slice(start, oldLines.length - endOffset);
    const newMiddle = newLines.slice(start, newLines.length - endOffset);
    lines.push(...diffMiddle(oldMiddle, newMiddle, start));

    for (let offset = endOffset; offset > 0; offset--) {
        const oldIndex = oldLines.length - offset;
        const newIndex = newLines.length - offset;
        lines.push({ kind: 'same', text: oldLines[oldIndex], oldNumber: oldIndex + 1, newNumber: newIndex + 1 });
    }
    return lines;
}

export function summarize(lines: DiffLine[]): DiffSummary {
    return {
        added: lines.filter((line) => line.kind === 'added').length,
        removed: lines.filter((line) => line.kind === 'removed').length
    };
}
