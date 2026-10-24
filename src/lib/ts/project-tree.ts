export const FOLDER_PLACEHOLDER = '.keep';

export interface ProjectFile {
    id: string;
    path: string;
    kind: string;
}

export interface FolderNode {
    type: 'folder';
    name: string;
    path: string;
    children: TreeNode[];
}

export interface FileNode {
    type: 'file';
    name: string;
    file: ProjectFile;
}

export type TreeNode = FolderNode | FileNode;

export function isPlaceholder(path: string) {
    return path === FOLDER_PLACEHOLDER || path.endsWith(`/${FOLDER_PLACEHOLDER}`);
}

export function placeholderPath(folder: string) {
    return `${folder}/${FOLDER_PLACEHOLDER}`;
}

function findOrCreateFolder(children: TreeNode[], name: string, path: string): FolderNode {
    const existing = children.find((node): node is FolderNode => node.type === 'folder' && node.name === name);
    if (existing) return existing;
    const folder: FolderNode = { type: 'folder', name, path, children: [] };
    children.push(folder);
    return folder;
}

function sortNodes(nodes: TreeNode[]) {
    nodes.sort((first, second) => {
        if (first.type !== second.type) return first.type === 'folder' ? -1 : 1;
        return first.name.localeCompare(second.name, undefined, { numeric: true, sensitivity: 'base' });
    });
    for (const node of nodes) {
        if (node.type === 'folder') sortNodes(node.children);
    }
}

export function buildProjectTree(files: ProjectFile[]): TreeNode[] {
    const root: TreeNode[] = [];
    for (const file of files) {
        const parts = file.path.split('/');
        let children = root;
        for (let index = 0; index < parts.length - 1; index++) {
            const folderPath = parts.slice(0, index + 1).join('/');
            children = findOrCreateFolder(children, parts[index], folderPath).children;
        }
        if (!isPlaceholder(file.path)) children.push({ type: 'file', name: parts[parts.length - 1], file });
    }
    sortNodes(root);
    return root;
}

export function projectFolders(files: ProjectFile[]): string[] {
    const folders = new Set<string>();
    for (const file of files) {
        const parts = file.path.split('/');
        for (let index = 1; index < parts.length; index++) folders.add(parts.slice(0, index).join('/'));
    }
    return [...folders].sort((first, second) => first.localeCompare(second));
}
