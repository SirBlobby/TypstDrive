<script lang="ts">
    import { onMount } from 'svelte';
    import { editorViewStore } from '#lib/ts/store';
    import { activeThreadStore, buildThreads, commentsStore } from '#lib/ts/comments';
    import { resolveRange } from '#lib/ts/document-anchors';
    import { activeSource, currentReviewTarget, inActiveFile, reviewTargetStore } from '#lib/ts/review-target';
    import { setCommentMarks, type CommentMark } from '#lib/ts/editor-comment-marks';

    let documentRevision = $state(0);

    function visibleMarks(): CommentMark[] {
        const target = $reviewTargetStore;
        const source = target ? activeSource(target) : null;
        if (!target || !source) return [];
        const marks: CommentMark[] = [];
        for (const thread of buildThreads($commentsStore)) {
            if (thread.root.resolved || !inActiveFile(target, thread.root)) continue;
            const range = resolveRange(source.doc, thread.root);
            if (!range || range.to <= range.from) continue;
            marks.push({
                id: thread.root.id,
                from: range.from,
                to: range.to,
                authorName: thread.root.author?.name ?? 'Someone',
                preview: thread.root.content,
                replyCount: thread.replies.length,
                active: thread.root.id === $activeThreadStore
            });
        }
        return marks;
    }

    onMount(() => {
        const target = currentReviewTarget();
        const sharedDoc = target ? activeSource(target)?.doc : null;
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
        const marks = visibleMarks();
        const documentLength = view.state.doc.length;
        view.dispatch({ effects: setCommentMarks.of(marks.filter((mark) => mark.to <= documentLength)) });
    });
</script>
