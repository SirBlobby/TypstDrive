import { uploadDocumentFile } from './document-uploads';

export interface LibraryImage {
    id: string;
    name: string;
    mime_type: string;
    size_bytes: number;
}

export interface PickerImage {
    key: string;
    name: string;
    section: string;
    sizeBytes: number | null;
    thumbnail: string;
    choose: () => Promise<string>;
}

export interface ImageLibrary {
    load: () => Promise<PickerImage[]>;
    upload: (file: File) => Promise<string>;
}

export interface ImageCodeFormat {
    defaultWidth: string;
    build: (path: string, width: string) => string;
}

export const typstImageFormat: ImageCodeFormat = {
    defaultWidth: '100%',
    build: (path, width) => `#image("${path}"${width ? `, width: ${width}` : ''})`
};

export const latexImageFormat: ImageCodeFormat = {
    defaultWidth: '0.8\\linewidth',
    build: (path, width) => `\\includegraphics${width ? `[width=${width}]` : ''}{${path}}`
};

interface ProjectFileEntry {
    id: string;
    path: string;
    kind: string;
}

const DRIVE_SECTION = 'Your drive';
const PROJECT_SECTION = 'In this project';
const IMAGE_PATH = /\.(png|jpe?g|gif|svg|webp|bmp)$/i;

async function fetchJson<T>(url: string, fallback: string): Promise<T> {
    const res = await fetch(url);
    if (!res.ok) throw new Error((await res.text()) || fallback);
    return res.json();
}

export function documentImageLibrary(docId: string): ImageLibrary {
    return {
        async load() {
            const images = await fetchJson<LibraryImage[]>(`/api/docs/${docId}/library/images`, 'Failed to load images.');
            return images.map((image) => ({
                key: image.id,
                name: image.name,
                section: DRIVE_SECTION,
                sizeBytes: image.size_bytes,
                thumbnail: `/api/docs/${docId}/files/${image.id}`,
                async choose() {
                    await fetch(`/api/docs/${docId}/files/${image.id}/attach`, { method: 'POST' }).catch(() => {});
                    return image.name;
                }
            }));
        },
        async upload(file) {
            const uploaded = await uploadDocumentFile(docId, file);
            if (!uploaded) throw new Error('The upload did not return a file name.');
            return uploaded.filename;
        }
    };
}

function freePath(name: string, taken: Set<string>) {
    if (!taken.has(name)) return name;
    const dot = name.lastIndexOf('.');
    const stem = dot > 0 ? name.slice(0, dot) : name;
    const extension = dot > 0 ? name.slice(dot) : '';
    let counter = 1;
    while (taken.has(`${stem}-${counter}${extension}`)) counter += 1;
    return `${stem}-${counter}${extension}`;
}

async function uploadToProject(projectId: string, file: Blob, name: string) {
    const form = new FormData();
    form.append('file', file, name);
    const res = await fetch(`/api/projects/${projectId}/files/upload`, { method: 'POST', body: form });
    if (!res.ok) throw new Error((await res.text()) || 'Upload failed.');
    const result: { files: string[] } = await res.json();
    return result.files[0] ?? name;
}

export function projectImageLibrary(projectId: string, onFilesChanged: () => void): ImageLibrary {
    let projectPaths = new Set<string>();

    async function copyFromDrive(image: LibraryImage) {
        const res = await fetch(`/api/files/${image.id}/data`);
        if (!res.ok) throw new Error('Could not read the image from your drive.');
        const path = await uploadToProject(projectId, await res.blob(), freePath(image.name, projectPaths));
        projectPaths.add(path);
        onFilesChanged();
        return path;
    }

    return {
        async load() {
            const [files, driveImages] = await Promise.all([
                fetchJson<ProjectFileEntry[]>(`/api/projects/${projectId}/files`, 'Failed to load project files.'),
                fetchJson<LibraryImage[]>(`/api/projects/${projectId}/library/images`, 'Failed to load images.')
            ]);
            projectPaths = new Set(files.map((file) => file.path));
            const projectImages: PickerImage[] = files
                .filter((file) => file.kind !== 'text' && IMAGE_PATH.test(file.path))
                .map((file) => ({
                    key: file.id,
                    name: file.path,
                    section: PROJECT_SECTION,
                    sizeBytes: null,
                    thumbnail: `/api/projects/${projectId}/files/${file.id}`,
                    choose: async () => file.path
                }));
            const drive: PickerImage[] = driveImages.map((image) => ({
                key: image.id,
                name: image.name,
                section: DRIVE_SECTION,
                sizeBytes: image.size_bytes,
                thumbnail: `/api/files/${image.id}/data`,
                choose: () => copyFromDrive(image)
            }));
            return [...projectImages, ...drive];
        },
        async upload(file) {
            const path = await uploadToProject(projectId, file, freePath(file.name, projectPaths));
            projectPaths.add(path);
            onFilesChanged();
            return path;
        }
    };
}
