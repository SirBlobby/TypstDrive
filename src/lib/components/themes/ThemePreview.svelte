<script lang="ts">
    import type { ThemePalette } from '#lib/ts/custom-themes';

    let { palette }: { palette: ThemePalette } = $props();
</script>

<div
    class="overflow-hidden rounded-md border text-[11px]"
    style="background: {palette.surface_muted}; border-color: {palette.line}; color: {palette.ink};"
>
    <div class="flex items-center justify-between px-3 py-2" style="background: {palette.surface}; border-bottom: 1px solid {palette.line};">
        <span class="font-semibold">Report.typ</span>
        <span class="rounded px-2 py-0.5 font-medium text-white" style="background: {palette.accent};">Share</span>
    </div>

    <div class="flex gap-2 p-2">
        <div class="w-16 flex-shrink-0 space-y-1">
            <div class="rounded px-1.5 py-1 font-medium" style="background: {palette.accent_soft}; color: {palette.accent};">Drive</div>
            <div class="px-1.5 py-1" style="color: {palette.ink_muted};">Shared</div>
            <div class="rounded px-1.5 py-1" style="background: {palette.surface_sunken}; color: {palette.ink_muted};">Trash</div>
        </div>

        <div class="min-w-0 flex-1 overflow-hidden rounded py-1.5 font-mono leading-5" style="background: {palette.editor_background}; color: {palette.editor_text};">
            <div class="px-2" style="color: {palette.comment};">// Notes</div>
            <div class="px-2">
                <span style="color: {palette.keyword};">#let</span>
                <span style="color: {palette.variable};">title</span> =
                <span style="color: {palette.string};">"Draft"</span>
            </div>
            <div class="px-2">
                <span style="color: {palette.keyword};">#set</span>
                <span style="color: {palette.function};">page</span>(margin:
                <span style="color: {palette.number};">2cm</span>)
            </div>
            <div class="px-2" style="background: {palette.active_line};">
                = <span style="background: {palette.selection};">Intro</span><span
                    class="ml-px inline-block h-3 w-px align-middle"
                    style="background: {palette.cursor};"
                ></span>
            </div>
        </div>
    </div>

    <div class="flex gap-3 px-3 pb-2">
        <span style="color: {palette.success};">Saved</span>
        <span style="color: {palette.danger};">1 error</span>
    </div>
</div>
