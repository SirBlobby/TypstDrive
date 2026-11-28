<script lang="ts">
    import { onMount } from 'svelte';
    import { editorViewStore } from '#lib/ts/store';
    import { userStore } from '#lib/ts/account/auth';
    import { resolveRange } from '#lib/ts/review/anchors';
    import { activeSource, currentReviewTarget, inActiveFile, reviewTargetStore, type ReviewSource } from '#lib/ts/review/target';
    import { remoteDrafts, remoteDraftsChangedStore, type RemoteDraft } from '#lib/ts/review/live';
    import { activeSuggestionStore, currentSuggestionRange, suggestingModeStore, suggestionsStore } from '#lib/ts/review/suggestions';
    import { endLiveDraft, LIVE_DRAFT_ID, liveDraftStore, suggestingHandlers } from '#lib/ts/review/suggestion-drafts';
    import { setSuggestionMarks, suggestingCompartment, type SuggestionMark } from '#lib/ts/editor/suggestion-marks';
    import { suggestingInput } from '#lib/ts/editor/suggesting-input';

    let documentRevision = $state(0);

    function liveDraftMark(): SuggestionMark | null {
        const target = $reviewTargetStore;
        const source = target ? activeSource(target) : null;
        const draft = $liveDraftStore;
        if (!source || !draft) return null;
        const range = resolveRange(source.doc, draft);
        if (!range) return null;
        return {
            id: draft.serverId ?? LIVE_DRAFT_ID,
            from: range.from,
            to: range.to,
            replacement: draft.replacement,
            authorName: $userStore?.username ?? 'You',
            note: null,
            active: true,
            caret: draft.caret
        };
    }

    function remoteDraftMarks(drafts: RemoteDraft[], source: ReviewSource): SuggestionMark[] {
        const marks: SuggestionMark[] = [];
        for (const draft of drafts) {
            const range = resolveRange(source.doc, draft);
            if (!range) continue;
            marks.push({ id: draft.id, ...range, replacement: draft.replacement, authorName: draft.authorName, note: null, active: false, caret: null });
        }
        return marks;
    }

    function pendingMarks(): SuggestionMark[] {
        $remoteDraftsChangedStore;
        const target = $reviewTargetStore;
        const source = target ? activeSource(target) : null;
        if (!target || !source) return [];
        const drafts = remoteDrafts(source.awareness);
        const liveIds = new Set([$liveDraftStore?.serverId, ...drafts.map((draft) => draft.serverId)].filter(Boolean));
        const marks: SuggestionMark[] = remoteDraftMarks(drafts, source);
        for (const suggestion of $suggestionsStore) {
            if (suggestion.status !== 'pending' || liveIds.has(suggestion.id) || !inActiveFile(target, suggestion)) continue;
            const range = currentSuggestionRange(source.doc, suggestion);
            if (!range) continue;
            marks.push({
                id: suggestion.id,
                from: range.from,
                to: range.to,
                replacement: suggestion.replacement_text,
                authorName: suggestion.author?.name ?? 'Someone',
                note: suggestion.note,
                active: suggestion.id === $activeSuggestionStore,
                caret: null
            });
        }
        const draftMark = liveDraftMark();
        return draftMark ? [...marks, draftMark] : marks;
    }

    onMount(() => {
        const target = currentReviewTarget();
        const sharedDoc = target ? activeSource(target)?.doc : null;
        if (!sharedDoc) return endLiveDraft;
        let timer: ReturnType<typeof setTimeout> | undefined;
        const refresh = () => {
            clearTimeout(timer);
            timer = setTimeout(() => documentRevision++, 400);
        };
        sharedDoc.on('update', refresh);
        return () => {
            clearTimeout(timer);
            sharedDoc.off('update', refresh);
            endLiveDraft();
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
        const suggesting = $suggestingModeStore;
        if (!view) return;
        if (!suggesting) endLiveDraft();
        view.dispatch({ effects: suggestingCompartment.reconfigure(suggesting ? suggestingInput(suggestingHandlers) : []) });
    });
</script>
