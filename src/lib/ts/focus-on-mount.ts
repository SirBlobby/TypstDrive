export function focusOnMount(node: HTMLElement) {
    requestAnimationFrame(() => node.focus());
}
