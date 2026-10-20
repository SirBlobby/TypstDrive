<script lang="ts">
    import { editorViewStore } from '$lib/ts/store';
    import { insertDialogStore } from '$lib/ts/insert-dialog';
    import { insertAtCursor, insertBlock } from '$lib/ts/editor-insert';
    import { symbolCategories, type PickerCharacter } from '$lib/ts/editor-symbols';
    import { emojiCategories } from '$lib/ts/editor-emoji';
    import InsertImageModal from './InsertImageModal.svelte';
    import InsertTableModal from './InsertTableModal.svelte';
    import InsertDocumentModal from './InsertDocumentModal.svelte';
    import CharacterPickerModal from './CharacterPickerModal.svelte';

    let { docId }: { docId: string } = $props();

    function close() {
        insertDialogStore.set(null);
    }

    function insertInline(insertion: string) {
        if ($editorViewStore) insertAtCursor($editorViewStore, insertion);
    }

    function insertAsBlock(block: string) {
        if ($editorViewStore) insertBlock($editorViewStore, block);
        close();
    }

    function pickCharacter(entry: PickerCharacter) {
        insertInline(entry.character);
    }
</script>

{#if $insertDialogStore === 'image'}
    <InsertImageModal {docId} onInsert={insertAsBlock} onClose={close} />
{:else if $insertDialogStore === 'table'}
    <InsertTableModal onInsert={insertAsBlock} onClose={close} />
{:else if $insertDialogStore === 'document'}
    <InsertDocumentModal currentDocId={docId} onInsert={insertAsBlock} onClose={close} />
{:else if $insertDialogStore === 'symbol'}
    <CharacterPickerModal title="Insert symbol" icon="mdi:omega" categories={symbolCategories} onPick={pickCharacter} onClose={close} />
{:else if $insertDialogStore === 'emoji'}
    <CharacterPickerModal title="Insert emoji" icon="mdi:emoticon-happy-outline" categories={emojiCategories} onPick={pickCharacter} onClose={close} />
{/if}
