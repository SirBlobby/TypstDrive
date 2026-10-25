<script lang="ts">
    import { onMount } from 'svelte';
    import { doc, text } from '#lib/ts/yjs-setup';
    import { editorViewStore } from '#lib/ts/store';
    import { anchorRange } from '#lib/ts/document-anchors';
    import {
        activeSuggestionStore,
        currentSuggestionRange,
        suggestingModeStore,
        suggestionDraftStore,
        suggestionsStore
    } from '#lib/ts/suggestions';
    import { setSuggestionMarks, suggestingCompartment, suggestingFilter, type CapturedEdit, type SuggestionMark } from '#lib/ts/editor-suggestions';

    let documentRevision = $state(0);

    function openDraft(edit: CapturedEdit) {
        if (!text) return;
        suggestionDraftStore.set({
            ...anchorRange(text, edit.from, edit.to),
            from: edit.from,
            to: edit.to,
            original_text: edit.original,
            replacement_text: edit.replacement
        });
    }

    function pendingMarks(): SuggestionMark[] {
        if (!doc) return [];
        const marks: SuggestionMark[] = [];
        for (const suggestion of $suggestionsStore) {
            if (suggestion.status !== 'pending') continue;
            const range = currentSuggestionRange(doc, suggestion);
            if (!range) continue;
            marks.push({
                id: suggestion.id,
                from: range.from,
                to: range.to,
                replacement: suggestion.replacement_text,
                authorName: suggestion.author?.name ?? 'Someone',
                note: suggestion.note,
                active: suggestion.id === $activeSuggestionStore
            });
        }
        return marks;
    }

    onMount(() => {
        const sharedDoc = doc;
        if (!sharedDoc) return;
        let timer: ReturnType<typeof setTimeout> | undefined;
        const refresh = () => {
            clearTimeout(timer);
            timer = setTimeout(() => documentRevision++, 400);
        };
        sharedDoc.on('update', refresh);
        return () => {
            clearTimeout(timer);
            sharedDoc.off('update', refresh);
        };
    });

    $effect(() => {
        documentRevision;
        const view = $editorViewStore;
        if (!view) return;
        const documentLength = view.state.doc.length;
        const marks = pendingMarks().filter((mark) => mark.to <= documentLength);
        view.dispatch({ effects: setSuggestionMarks.of(marks) });
    });

    $effect(() => {
        const view = $editorViewStore;
        if (!view) return;
        view.dispatch({ effects: suggestingCompartment.reconfigure($suggestingModeStore ? suggestingFilter(openDraft) : []) });
    });
</script>
