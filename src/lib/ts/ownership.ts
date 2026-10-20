export type ShareKind = 'document' | 'project' | 'folder';

export interface OwnershipTransfer {
    id: string;
    item_kind: ShareKind;
    item_id: string;
    item_name?: string;
    from_user_id: string;
    from_username: string;
    to_user_id: string;
    to_username: string;
    created_at: string;
}

export interface ShareInfo {
    is_owner: boolean;
    pending_transfer: OwnershipTransfer | null;
}

async function errorText(res: Response, fallback: string) {
    return (await res.text()) || fallback;
}

export async function fetchShareInfo(kind: ShareKind, itemId: string): Promise<ShareInfo | null> {
    try {
        const res = await fetch(`/api/share-info/${kind}/${itemId}`);
        return res.ok ? await res.json() : null;
    } catch {
        return null;
    }
}

export async function fetchTransfers(): Promise<{ incoming: OwnershipTransfer[]; outgoing: OwnershipTransfer[] }> {
    try {
        const res = await fetch('/api/transfers');
        if (res.ok) return await res.json();
    } catch {}
    return { incoming: [], outgoing: [] };
}

export async function requestTransfer(kind: ShareKind, itemId: string, toUserId: string): Promise<OwnershipTransfer> {
    const res = await fetch('/api/transfers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ item_kind: kind, item_id: itemId, to_user_id: toUserId })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to start the transfer.'));
    return res.json();
}

export async function acceptTransfer(transferId: string) {
    const res = await fetch(`/api/transfers/${transferId}/accept`, { method: 'POST' });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to accept the transfer.'));
}

export async function cancelTransfer(transferId: string) {
    const res = await fetch(`/api/transfers/${transferId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to cancel the transfer.'));
}
