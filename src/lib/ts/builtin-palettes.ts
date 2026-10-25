import { themes, type ThemeColors } from './themes';
import type { ThemeDraft, ThemeMode, ThemePalette } from './custom-themes';

type AppColors = Pick<ThemePalette, 'surface' | 'surface_muted' | 'surface_sunken' | 'line' | 'ink' | 'ink_muted' | 'accent' | 'accent_soft' | 'danger' | 'success'>;

const appColors: Record<string, Record<ThemeMode, AppColors>> = {
    Cerberus: {
        light: {
            surface: '#ffffff', surface_muted: '#f8f8fa', surface_sunken: '#ececf0', line: '#e2e2e8', ink: '#18181b',
            ink_muted: '#5f5f6b', accent: '#4f46e5', accent_soft: '#eceafd', danger: '#dc2626', success: '#15803d'
        },
        dark: {
            surface: '#121212', surface_muted: '#171717', surface_sunken: '#1f1f1f', line: '#262626', ink: '#f5f5f5',
            ink_muted: '#737373', accent: '#818cf8', accent_soft: '#262244', danger: '#f4736a', success: '#4cc47f'
        }
    },
    Catppuccin: {
        light: {
            surface: '#eff1f5', surface_muted: '#e6e9ef', surface_sunken: '#dce0e8', line: '#ccd0da', ink: '#4c4f69',
            ink_muted: '#6c6f85', accent: '#1e66f5', accent_soft: '#d6e2fd', danger: '#d20f39', success: '#40a02b'
        },
        dark: {
            surface: '#181825', surface_muted: '#1e1e2e', surface_sunken: '#45475a', line: '#313244', ink: '#cdd6f4',
            ink_muted: '#6c7086', accent: '#89b4fa', accent_soft: '#1e2f4d', danger: '#f4736a', success: '#4cc47f'
        }
    },
    'Arch Linux': {
        light: {
            surface: '#ffffff', surface_muted: '#f5f7f9', surface_sunken: '#e8edf1', line: '#d5dde3', ink: '#1f2328',
            ink_muted: '#59636e', accent: '#0a6ea8', accent_soft: '#e1f1fa', danger: '#cf222e', success: '#1a7f37'
        },
        dark: {
            surface: '#010409', surface_muted: '#0d1117', surface_sunken: '#161b22', line: '#21262d', ink: '#c9d1d9',
            ink_muted: '#6e7681', accent: '#1793d1', accent_soft: '#0d2b3d', danger: '#f4736a', success: '#4cc47f'
        }
    }
};

function editorPalette(colors: ThemeColors) {
    return {
        editor_background: colors.background,
        editor_text: colors.text,
        selection: colors.selection,
        active_line: colors.activeLine,
        cursor: colors.cursor,
        keyword: colors.keyword,
        string: colors.string,
        number: colors.number,
        comment: colors.comment,
        variable: colors.variable,
        function: colors.function
    };
}

export const builtinThemeNames = Object.keys(appColors);

export function builtinThemeDraft(name: string): ThemeDraft {
    const themeName = appColors[name] ? name : 'Cerberus';
    return {
        name: '',
        light: { ...appColors[themeName].light, ...editorPalette(themes[themeName].light) },
        dark: { ...appColors[themeName].dark, ...editorPalette(themes[themeName].dark) }
    };
}
