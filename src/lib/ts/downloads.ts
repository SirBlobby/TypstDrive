function startDownload(url: string) {
    const link = document.createElement('a');
    link.href = url;
    link.rel = 'noopener';
    document.body.appendChild(link);
    link.click();
    link.remove();
}

export function downloadFolder(folderId: string) {
    startDownload(`/api/folders/${folderId}/download`);
}

export function downloadProject(projectId: string) {
    startDownload(`/api/projects/${projectId}/download`);
}
