export type MinimapSize = 'small' | 'medium' | 'large';

export interface MinimapMetrics {
    width: number;
    lineHeight: number;
}

export const minimapMetrics: Record<MinimapSize, MinimapMetrics> = {
    small: { width: 140, lineHeight: 6 },
    medium: { width: 190, lineHeight: 8 },
    large: { width: 240, lineHeight: 10 }
};

export const minimapSizeOptions: { value: MinimapSize; label: string }[] = [
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' }
];

export function isMinimapSize(value: unknown): value is MinimapSize {
    return typeof value === 'string' && value in minimapMetrics;
}
