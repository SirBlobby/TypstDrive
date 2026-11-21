export type PreviewKind = 'image' | 'pdf' | 'none';

export const MIN_ZOOM = 0.25;
export const MAX_ZOOM = 4;
export const ZOOM_STEP = 0.25;

export function previewKindFor(mimeType: string | null | undefined, name: string | null | undefined): PreviewKind {
    const mime = (mimeType ?? '').toLowerCase();
    const lowerName = (name ?? '').toLowerCase();
    if (mime.startsWith('image/')) return 'image';
    if (mime === 'application/pdf' || lowerName.endsWith('.pdf')) return 'pdf';
    return 'none';
}

export function stepZoom(zoom: number, step: number) {
    return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, Math.round((zoom + step) * 100) / 100));
}

// Same id can hold new bytes after a re-upload, so bust the browser cache on change.
export function versionedUrl(url: string, updatedAt: string | null | undefined) {
    return updatedAt ? `${url}?v=${encodeURIComponent(updatedAt)}` : url;
}

export function projectFileUrl(projectId: string, file: { id: string; updated_at?: string | null }) {
    return versionedUrl(`/api/projects/${projectId}/files/${file.id}`, file.updated_at);
}
