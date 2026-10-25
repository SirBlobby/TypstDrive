<script lang="ts">
    import { onMount } from 'svelte';
    import { doc } from '#lib/ts/yjs-setup';
    import { editorViewStore } from '#lib/ts/store';
    import { activeThreadStore, buildThreads, commentsStore } from '#lib/ts/comments';
    import { resolveRange } from '#lib/ts/document-anchors';
    import { setCommentMarks, type CommentMark } from '#lib/ts/editor-comment-marks';

    let documentRevision = $state(0);

    function visibleMarks(): CommentMark[] {
        if (!doc) return [];
        const marks: CommentMark[] = [];
        for (const thread of buildThreads($commentsStore)) {
            if (thread.root.resolved) continue;
            const range = resolveRange(doc, thread.root);
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
        const marks = visibleMarks();
        const documentLength = view.state.doc.length;
        view.dispatch({ effects: setCommentMarks.of(marks.filter((mark) => mark.to <= documentLength)) });
    });
</script>
