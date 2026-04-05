<script lang="ts">
	import type { Diagnostic } from '../ts/typst-api';

	let { errors = [] } = $props<{ errors: Diagnostic[] }>();
</script>

{#if errors && errors.length > 0}
	<div class="absolute bottom-4 left-4 right-4 z-50 rounded-lg shadow-2xl bg-red-900/90 text-white p-4 max-h-48 overflow-y-auto backdrop-blur-sm border border-red-500/50">
		<h3 class="font-bold flex items-center gap-2 mb-2">
			<svg xmlns="http://www.w3.org/2000/svg" width="1.2em" height="1.2em" viewBox="0 0 24 24"><path fill="currentColor" d="M12 17q.425 0 .713-.288T13 16t-.288-.712T12 15t-.712.288T11 16t.288.713T12 17m-1-4h2V7h-2zm1 9q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22"/></svg>
			Compilation Errors
		</h3>
		<ul class="text-sm font-mono flex flex-col gap-1">
			{#each errors as error}
				<li class="flex items-start gap-2">
					<span class="text-red-300">[{error.severity}]</span>
					<span>{error.message}</span>
				</li>
			{/each}
		</ul>
	</div>
{/if}
