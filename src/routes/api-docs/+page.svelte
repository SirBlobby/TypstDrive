<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/stores';
    import Icon from '@iconify/svelte';
    import Footer from '$lib/components/Footer.svelte';
    import hljs from 'highlight.js/lib/core';
    import bash from 'highlight.js/lib/languages/bash';
    import javascript from 'highlight.js/lib/languages/javascript';
    import python from 'highlight.js/lib/languages/python';
    import json from 'highlight.js/lib/languages/json';

    hljs.registerLanguage('bash', bash);
    hljs.registerLanguage('javascript', javascript);
    hljs.registerLanguage('python', python);
    hljs.registerLanguage('json', json);

    let activeSection = $state('overview');
    let copiedSnippet = $state<string | null>(null);

    let baseUrl = $derived($page.url.origin);

    let curlPng = $derived(`curl -X POST ${baseUrl}/v1/render \\
  -H "Authorization: Bearer td_your_api_key_here" \\
  -H "Content-Type: application/json" \\
  -d '{"code":"#set page(width:200pt,height:80pt)\\nHello, *World*!","format":"png"}' \\
  --output hello.png`);

    let curlPdf = $derived(`curl -X POST ${baseUrl}/v1/render \\
  -H "Authorization: Bearer td_your_api_key_here" \\
  -H "Content-Type: application/json" \\
  -d '{"code":"= My Report\\n\\nSome body text.","format":"pdf"}' \\
  --output report.pdf`);

    let jsExample = $derived(`const response = await fetch('${baseUrl}/v1/render', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer td_your_api_key_here',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    code: '#set page(width: 200pt, height: 80pt)\\nHello, *World*!',
    format: 'png',
  }),
});

if (!response.ok) throw new Error(await response.text());

const blob = await response.blob();
document.querySelector('img').src = URL.createObjectURL(blob);`);

    let pythonExample = $derived(`import httpx

response = httpx.post(
    "${baseUrl}/v1/render",
    headers={"Authorization": "Bearer td_your_api_key_here"},
    json={
        "code": "= My Report\\\\n\\\\nSome body text.",
        "format": "pdf",
    },
)
response.raise_for_status()

with open("report.pdf", "wb") as f:
    f.write(response.content)`);

    let filesExample = $derived(`import base64, httpx

with open("logo.png", "rb") as f:
    logo_b64 = base64.b64encode(f.read()).decode()

response = httpx.post(
    "${baseUrl}/v1/render",
    headers={"Authorization": "Bearer td_your_api_key_here"},
    json={
        "code": """
#set page(width: 300pt, height: 200pt)
#image("logo.png", width: 80pt)
= My Report
Some body text.
""",
        "format": "png",
        "files": [{"name": "logo.png", "data": logo_b64}],
    },
)
response.raise_for_status()
with open("output.png", "wb") as f:
    f.write(response.content)`);

    const requestSchemaJson = `{
  "code":   "string",          // Typst markup (required)
  "format": "png" | "pdf",    // Output format (required)
  "files":  [                  // Optional inline assets
    {
      "name": "string",        // Filename used in Typst code
      "data": "string"         // Base64-encoded file content
    }
  ]
}`;

    // Highlighted versions (derived so they update if baseUrl changes)
    let hCurlPng    = $derived(hljs.highlight(curlPng,          { language: 'bash'       }).value);
    let hCurlPdf    = $derived(hljs.highlight(curlPdf,          { language: 'bash'       }).value);
    let hJs         = $derived(hljs.highlight(jsExample,        { language: 'javascript' }).value);
    let hPython     = $derived(hljs.highlight(pythonExample,    { language: 'python'     }).value);
    let hFiles      = $derived(hljs.highlight(filesExample,     { language: 'python'     }).value);
    let hSchema     = $derived(hljs.highlight(requestSchemaJson,{ language: 'json'       }).value);

    async function copy(id: string, text: string) {
        await navigator.clipboard.writeText(text);
        copiedSnippet = id;
        setTimeout(() => copiedSnippet = null, 2000);
    }

    const navSections = [
        { id: 'overview',    label: 'Overview',        icon: 'mdi:book-open-outline'      },
        { id: 'auth',        label: 'Authentication',  icon: 'mdi:key-outline'            },
        { id: 'endpoint',    label: 'POST /v1/render', icon: 'mdi:api'                    },
        { id: 'examples',    label: 'Examples',        icon: 'mdi:code-braces'            },
        { id: 'rate-limits', label: 'Rate Limits',     icon: 'mdi:speedometer'            },
        { id: 'errors',      label: 'Error Reference', icon: 'mdi:alert-circle-outline'   },
    ];
</script>

<svelte:head>
    <title>API Docs - TypstDrive</title>
    <meta name="description" content="TypstDrive Render API documentation." />
</svelte:head>

<style>
    :global(.hljs) {
        color: #abb2bf;
        background: #1e2127;
    }
    :global(.hljs-comment), :global(.hljs-quote) { color: #5c6370; font-style: italic; }
    :global(.hljs-doctag), :global(.hljs-keyword), :global(.hljs-formula) { color: #c678dd; }
    :global(.hljs-section), :global(.hljs-name), :global(.hljs-selector-tag),
    :global(.hljs-deletion), :global(.hljs-subst) { color: #e06c75; }
    :global(.hljs-literal) { color: #56b6c2; }
    :global(.hljs-string), :global(.hljs-regexp), :global(.hljs-addition),
    :global(.hljs-attribute), :global(.hljs-meta .hljs-string) { color: #98c379; }
    :global(.hljs-attr), :global(.hljs-variable), :global(.hljs-template-variable),
    :global(.hljs-type), :global(.hljs-selector-class), :global(.hljs-selector-attr),
    :global(.hljs-selector-pseudo), :global(.hljs-number) { color: #d19a66; }
    :global(.hljs-symbol), :global(.hljs-bullet), :global(.hljs-link),
    :global(.hljs-meta), :global(.hljs-selector-id), :global(.hljs-title) { color: #61aeee; }
    :global(.hljs-built_in), :global(.hljs-title.class_), :global(.hljs-class .hljs-title) { color: #e6c07b; }
    :global(.hljs-emphasis) { font-style: italic; }
    :global(.hljs-strong) { font-weight: bold; }
    :global(.hljs-link) { text-decoration: underline; }
</style>

<div class="min-h-screen flex flex-col">
    <nav class="bg-[var(--theme-bg)] shadow-sm border-b border-gray-200 dark:border-white/10 px-6 py-4 flex justify-between items-center sticky top-0 z-10 transition-colors duration-200 flex-shrink-0">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
            <Icon icon="mdi:api" class="text-blue-600 dark:text-blue-400 text-3xl" />
            API Reference
        </h1>
        <button onclick={() => goto('/dashboard')} class="text-sm font-medium text-gray-600 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-4 py-2 rounded-lg flex items-center gap-2">
            <Icon icon="mdi:arrow-left" class="text-lg" />
            Back to Dashboard
        </button>
    </nav>

    <div class="flex flex-1 max-w-6xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 gap-8">
        <aside class="w-56 flex-shrink-0 hidden md:block">
            <nav class="sticky top-24 space-y-1">
                {#each navSections as section}
                    <button
                        onclick={() => activeSection = section.id}
                        class="w-full flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 {activeSection === section.id
                            ? 'bg-blue-600 text-white shadow-sm'
                            : 'text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/5'}"
                    >
                        <Icon icon={section.icon} class="text-lg flex-shrink-0" />
                        {section.label}
                    </button>
                {/each}
                <div class="pt-4 mt-4 border-t border-gray-200 dark:border-white/10">
                    <a href="/settings" class="w-full flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/5 transition-all duration-150">
                        <Icon icon="mdi:key-plus" class="text-lg flex-shrink-0" />
                        Manage API Keys
                    </a>
                </div>
            </nav>
        </aside>

        <main class="flex-1 min-w-0 space-y-6 pb-16">

            <div class="md:hidden flex gap-2 flex-wrap">
                {#each navSections as section}
                    <button
                        onclick={() => activeSection = section.id}
                        class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors {activeSection === section.id ? 'bg-blue-600 text-white' : 'bg-gray-100 dark:bg-white/10 text-gray-600 dark:text-gray-300'}"
                    >
                        {section.label}
                    </button>
                {/each}
            </div>

            {#if activeSection === 'overview'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 p-6 sm:p-8">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                        <Icon icon="mdi:book-open-outline" class="text-2xl text-blue-500" />
                        Overview
                    </h2>
                    <p class="text-gray-600 dark:text-gray-300 mb-6">
                        The TypstDrive Render API lets you compile Typst markup into PNG images or PDF documents programmatically.
                        Authenticate with an API key and POST Typst code — get back binary output.
                    </p>
                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-6">
                        <div class="p-4 rounded-xl bg-blue-50 dark:bg-blue-900/10 border border-blue-100 dark:border-blue-800/30">
                            <Icon icon="mdi:image-outline" class="text-2xl text-blue-500 mb-2" />
                            <p class="text-sm font-semibold text-gray-800 dark:text-gray-200">PNG output</p>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">First page rendered at 2× scale</p>
                        </div>
                        <div class="p-4 rounded-xl bg-purple-50 dark:bg-purple-900/10 border border-purple-100 dark:border-purple-800/30">
                            <Icon icon="mdi:file-pdf-box" class="text-2xl text-purple-500 mb-2" />
                            <p class="text-sm font-semibold text-gray-800 dark:text-gray-200">PDF output</p>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Full multi-page PDF document</p>
                        </div>
                        <div class="p-4 rounded-xl bg-green-50 dark:bg-green-900/10 border border-green-100 dark:border-green-800/30">
                            <Icon icon="mdi:lightning-bolt" class="text-2xl text-green-500 mb-2" />
                            <p class="text-sm font-semibold text-gray-800 dark:text-gray-200">Cached results</p>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Identical inputs skip recompilation</p>
                        </div>
                    </div>
                    <div class="bg-gray-50 dark:bg-black/30 rounded-xl p-4 border border-gray-200 dark:border-white/10">
                        <p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">Base URL</p>
                        <code class="font-mono text-sm text-blue-600 dark:text-blue-400">{baseUrl}</code>
                    </div>
                </div>
            {/if}

            {#if activeSection === 'auth'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 p-6 sm:p-8">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                        <Icon icon="mdi:key-outline" class="text-2xl text-blue-500" />
                        Authentication
                    </h2>
                    <p class="text-gray-600 dark:text-gray-300 mb-6">
                        All requests must include an API key in the <code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-1.5 py-0.5 rounded">Authorization</code> header.
                    </p>
                    <div class="space-y-4">
                        <div>
                            <p class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">Header format</p>
                            <pre class="rounded-xl border border-gray-700 overflow-x-auto"><code class="hljs block px-4 py-3 text-xs font-mono leading-relaxed rounded-xl">{@html hljs.highlight('Authorization: Bearer td_your_api_key_here', { language: 'bash' }).value}</code></pre>
                        </div>
                        <div class="p-4 rounded-xl bg-amber-50 dark:bg-amber-900/10 border border-amber-200 dark:border-amber-700/30 flex gap-3">
                            <Icon icon="mdi:information-outline" class="text-amber-500 text-xl flex-shrink-0 mt-0.5" />
                            <div class="text-sm text-amber-800 dark:text-amber-300">
                                <p class="font-semibold mb-1">Keep your keys secret</p>
                                <p>API keys grant access to your account's uploaded files during compilation. Never expose them in client-side code or commit them to version control.</p>
                            </div>
                        </div>
                        <div>
                            <p class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1">Managing keys</p>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                Create, regenerate, and revoke keys in
                                <a href="/settings" class="text-blue-600 dark:text-blue-400 hover:underline">Settings → API Keys</a>.
                                The full key is shown only once at creation time.
                            </p>
                        </div>
                    </div>
                </div>
            {/if}

            {#if activeSection === 'endpoint'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 p-6 sm:p-8 space-y-6">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                        <Icon icon="mdi:api" class="text-2xl text-blue-500" />
                        POST /v1/render
                    </h2>

                    <div>
                        <div class="flex items-center gap-2 mb-3">
                            <span class="px-2 py-0.5 text-xs font-bold bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded-md">POST</span>
                            <code class="font-mono text-sm text-gray-800 dark:text-gray-200">/v1/render</code>
                        </div>
                        <p class="text-sm text-gray-600 dark:text-gray-400">
                            Compile Typst markup and return rendered binary output as PNG or PDF.
                            Results are cached for 1 hour — identical inputs return the cached result without recompiling.
                        </p>
                    </div>

                    <div class="h-px bg-gray-200 dark:bg-white/10"></div>

                    <div>
                        <p class="text-sm font-bold text-gray-800 dark:text-gray-200 mb-3">Request headers</p>
                        <div class="overflow-x-auto">
                            <table class="w-full text-sm">
                                <thead>
                                    <tr class="border-b border-gray-200 dark:border-white/10">
                                        <th class="text-left py-2 pr-4 font-semibold text-gray-700 dark:text-gray-300 w-40">Header</th>
                                        <th class="text-left py-2 font-semibold text-gray-700 dark:text-gray-300">Value</th>
                                    </tr>
                                </thead>
                                <tbody class="text-gray-600 dark:text-gray-400">
                                    <tr class="border-b border-gray-100 dark:border-white/5">
                                        <td class="py-2 pr-4 font-mono text-xs">Authorization</td>
                                        <td class="py-2"><code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-1.5 py-0.5 rounded">Bearer &lt;api-key&gt;</code> — required</td>
                                    </tr>
                                    <tr>
                                        <td class="py-2 pr-4 font-mono text-xs">Content-Type</td>
                                        <td class="py-2"><code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-1.5 py-0.5 rounded">application/json</code> — required</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>

                    <div>
                        <p class="text-sm font-bold text-gray-800 dark:text-gray-200 mb-3">Request body</p>
                        <pre class="rounded-xl border border-gray-700 overflow-x-auto"><code class="hljs block px-4 py-4 text-xs font-mono leading-relaxed rounded-xl">{@html hSchema}</code></pre>
                    </div>

                    <div>
                        <p class="text-sm font-bold text-gray-800 dark:text-gray-200 mb-3">Response</p>
                        <div class="p-3 rounded-lg bg-green-50 dark:bg-green-900/10 border border-green-100 dark:border-green-800/30 text-sm">
                            <span class="font-mono text-xs font-bold text-green-700 dark:text-green-400">200 OK</span>
                            <span class="text-gray-600 dark:text-gray-400 ml-2">Binary body with <code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-2 rounded">Content-Type: image/png</code> or <code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-2 rounded">application/pdf</code></span>
                        </div>
                    </div>

                    <div class="p-4 rounded-xl bg-blue-50 dark:bg-blue-900/10 border border-blue-100 dark:border-blue-800/30 text-sm text-blue-800 dark:text-blue-300">
                        <p class="font-semibold mb-1 flex items-center gap-2"><Icon icon="mdi:folder-account-outline" class="text-base" /> Account files available automatically</p>
                        <p>Files uploaded to your TypstDrive account are available by filename inside your Typst code. Pass additional files inline via the <code class="font-mono text-xs bg-blue-100 dark:bg-blue-800/40 px-1 rounded">files</code> array to supplement or override them.</p>
                    </div>
                </div>
            {/if}

            {#if activeSection === 'examples'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 p-6 sm:p-8 space-y-8">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                        <Icon icon="mdi:code-braces" class="text-2xl text-blue-500" />
                        Examples
                    </h2>

                    {#each [
                        { id: 'curl-png', label: 'cURL — render PNG',         icon: 'mdi:bash',                iconColor: 'text-gray-400', code: hCurlPng,  raw: curlPng     },
                        { id: 'curl-pdf', label: 'cURL — render PDF',         icon: 'mdi:bash',                iconColor: 'text-gray-400', code: hCurlPdf,  raw: curlPdf     },
                        { id: 'js',       label: 'JavaScript / TypeScript',   icon: 'mdi:language-javascript', iconColor: 'text-yellow-400', code: hJs,      raw: jsExample   },
                        { id: 'python',   label: 'Python (httpx)',             icon: 'mdi:language-python',     iconColor: 'text-blue-400',  code: hPython,  raw: pythonExample},
                        { id: 'files',    label: 'Python — with inline files', icon: 'mdi:file-image-outline',  iconColor: 'text-purple-400', code: hFiles,  raw: filesExample },
                    ] as ex}
                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <p class="text-sm font-bold text-gray-800 dark:text-gray-200 flex items-center gap-2">
                                    <Icon icon={ex.icon} class="text-lg {ex.iconColor}" />
                                    {ex.label}
                                </p>
                                <button onclick={() => copy(ex.id, ex.raw)} class="flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors px-2 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-white/10">
                                    <Icon icon={copiedSnippet === ex.id ? 'mdi:check' : 'mdi:content-copy'} class="text-sm" />
                                    {copiedSnippet === ex.id ? 'Copied!' : 'Copy'}
                                </button>
                            </div>
                            <pre class="rounded-xl border border-gray-700 overflow-x-auto"><code class="hljs block px-4 py-4 text-xs font-mono leading-relaxed rounded-xl">{@html ex.code}</code></pre>
                        </div>
                    {/each}
                </div>
            {/if}

            {#if activeSection === 'rate-limits'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 p-6 sm:p-8">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                        <Icon icon="mdi:speedometer" class="text-2xl text-blue-500" />
                        Rate Limits
                    </h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
                        <div class="p-4 rounded-xl bg-gray-50 dark:bg-black/30 border border-gray-200 dark:border-white/10">
                            <p class="text-2xl font-bold text-gray-900 dark:text-white">60</p>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">requests / minute per key</p>
                        </div>
                        <div class="p-4 rounded-xl bg-gray-50 dark:bg-black/30 border border-gray-200 dark:border-white/10">
                            <p class="text-2xl font-bold text-gray-900 dark:text-white">10</p>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">API keys per account</p>
                        </div>
                    </div>
                    <div class="p-4 rounded-xl bg-amber-50 dark:bg-amber-900/10 border border-amber-200 dark:border-amber-700/30 text-sm text-amber-800 dark:text-amber-300 mb-4">
                        <p class="font-semibold mb-1">Caching saves quota</p>
                        <p>Identical inputs (same code + files) skip recompilation and are served from cache for up to 1 hour. Cached responses return instantly and do not consume your rate limit.</p>
                    </div>
                    <div>
                        <p class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">When exceeded</p>
                        <div class="p-3 rounded-lg bg-red-50 dark:bg-red-900/10 border border-red-100 dark:border-red-800/30 text-sm">
                            <code class="font-mono text-xs font-bold text-red-700 dark:text-red-400">429 Too Many Requests</code>
                            <span class="text-gray-600 dark:text-gray-400 ml-2">— wait for the current 60-second window to reset.</span>
                        </div>
                    </div>
                </div>
            {/if}

            {#if activeSection === 'errors'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 p-6 sm:p-8">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                        <Icon icon="mdi:alert-circle-outline" class="text-2xl text-blue-500" />
                        Error Reference
                    </h2>
                    <div class="space-y-3">
                        {#each [
                            { code: '400', name: 'Bad Request',           desc: 'Invalid format value, empty code, or malformed JSON body.'                              },
                            { code: '401', name: 'Unauthorized',          desc: 'Missing or invalid Authorization header, or unknown API key.'                           },
                            { code: '422', name: 'Unprocessable Entity',  desc: 'Your Typst code compiled with errors. Fix the markup and retry.'                        },
                            { code: '429', name: 'Too Many Requests',     desc: 'Rate limit exceeded. Wait for the current 60-second window to reset.'                   },
                            { code: '500', name: 'Internal Server Error', desc: 'Unexpected server error. Try again after a short delay.'                                },
                        ] as err}
                            <div class="flex items-start gap-4 p-4 rounded-xl border border-gray-100 dark:border-white/10 bg-gray-50 dark:bg-black/20">
                                <code class="font-mono text-sm font-bold text-gray-800 dark:text-gray-200 flex-shrink-0 w-8">{err.code}</code>
                                <div>
                                    <p class="text-sm font-semibold text-gray-800 dark:text-gray-200">{err.name}</p>
                                    <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{err.desc}</p>
                                </div>
                            </div>
                        {/each}
                    </div>
                    <div class="mt-6 p-4 rounded-xl bg-gray-50 dark:bg-black/30 border border-gray-200 dark:border-white/10">
                        <p class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1">Error body</p>
                        <p class="text-sm text-gray-500 dark:text-gray-400">Error responses return plain text describing the issue — no JSON envelope.</p>
                    </div>
                </div>
            {/if}

        </main>
    </div>

    <Footer />
</div>
