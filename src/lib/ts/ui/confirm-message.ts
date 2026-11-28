export interface MessagePart {
    text: string;
    strong: boolean;
}

export interface ConfirmMessage {
    lead: MessagePart[];
    detail: string;
}

const QUOTED_NAME = /(^|\s)'([^']+)'(?=[\s?.!,:;]|$)/g;
const SENTENCE_END = /[.?!](?=\s)/;
const PLACEHOLDER = '\u0000';

function highlightNames(text: string, names: string[]): MessagePart[] {
    const parts: MessagePart[] = [];
    text.split(new RegExp(`${PLACEHOLDER}(\\d+)${PLACEHOLDER}`)).forEach((piece, index) => {
        if (index % 2 === 1) parts.push({ text: names[Number(piece)], strong: true });
        else if (piece) parts.push({ text: piece, strong: false });
    });
    return parts;
}

export function splitConfirmMessage(message: string): ConfirmMessage {
    const names: string[] = [];
    const masked = message.trim().replace(QUOTED_NAME, (_match, space: string, name: string) => {
        names.push(name);
        return `${space}${PLACEHOLDER}${names.length - 1}${PLACEHOLDER}`;
    });
    const end = masked.search(SENTENCE_END);
    const leadText = end === -1 ? masked : masked.slice(0, end + 1);
    const detailText = end === -1 ? '' : masked.slice(end + 1).trim();
    const restore = (text: string) => text.replace(new RegExp(`${PLACEHOLDER}(\\d+)${PLACEHOLDER}`, 'g'), (_match, index: string) => `'${names[Number(index)]}'`);
    return { lead: highlightNames(leadText, names), detail: restore(detailText) };
}
