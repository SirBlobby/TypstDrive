export type CropArea = {
    x: number;
    y: number;
    width: number;
    height: number;
};

function loadImage(source: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
        const image = new Image();
        image.onload = () => resolve(image);
        image.onerror = () => reject(new Error('Could not read the image.'));
        image.src = source;
    });
}

export async function cropImage(source: string, area: CropArea, maxSize: number): Promise<Blob> {
    const image = await loadImage(source);
    const scale = Math.min(1, maxSize / Math.max(area.width, area.height));
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(area.width * scale));
    canvas.height = Math.max(1, Math.round(area.height * scale));

    const context = canvas.getContext('2d');
    if (!context) throw new Error('Could not prepare the image.');
    context.drawImage(image, area.x, area.y, area.width, area.height, 0, 0, canvas.width, canvas.height);

    return new Promise((resolve, reject) => {
        canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error('Could not prepare the image.'))), 'image/png');
    });
}
