<script lang="ts">
    let { files, value = $bindable(), defaultName, label = 'Bibliography file', preferExisting = true }: {
        files: string[];
        value: string;
        defaultName: string;
        label?: string;
        preferExisting?: boolean;
    } = $props();

    const NEW_FILE = '__new__';

    let choice = $state(NEW_FILE);
    let newName = $state('');
    let pickedExisting = false;

    $effect(() => {
        if (preferExisting && !pickedExisting && files.length > 0) {
            pickedExisting = true;
            choice = files[0];
        }
    });

    $effect(() => {
        value = choice === NEW_FILE ? newName.trim() || defaultName : choice;
    });
</script>

<label class="flex flex-col gap-1.5 text-xs">
    <span class="font-medium text-[var(--color-ink-muted)]">{label}</span>
    <div class="flex gap-2">
        <select bind:value={choice} class="min-w-0 flex-1 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none">
            {#each files as file (file)}
                <option value={file}>{file}</option>
            {/each}
            <option value={NEW_FILE}>New file...</option>
        </select>
        {#if choice === NEW_FILE}
            <input bind:value={newName} placeholder={defaultName} class="min-w-0 flex-1 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 font-mono text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
        {/if}
    </div>
</label>
