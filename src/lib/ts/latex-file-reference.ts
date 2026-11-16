const GRAPHICS = /\.(png|jpe?g|pdf|eps)$/i;
const FONTS = /\.(ttf|otf)$/i;
const UNSUPPORTED_IMAGES = /\.(svg|gif|webp|bmp)$/i;

function stem(fileName: string) {
    return fileName.replace(/\.[^./]+$/, '');
}

export function latexFileReference(fileName: string) {
    if (GRAPHICS.test(fileName)) return `\\includegraphics[width=0.8\\linewidth]{${fileName}}\n`;
    if (FONTS.test(fileName)) return `\\setmainfont{${fileName}}\n`;
    if (UNSUPPORTED_IMAGES.test(fileName)) return `% LaTeX cannot show ${fileName}. Convert it to PNG, JPG or PDF first.\n`;
    if (/\.bib$/i.test(fileName)) return `\\bibliographystyle{plain}\n\\bibliography{${stem(fileName)}}\n`;
    if (/\.tex$/i.test(fileName)) return `\\input{${stem(fileName)}}\n`;
    return `% ${fileName}\n`;
}
