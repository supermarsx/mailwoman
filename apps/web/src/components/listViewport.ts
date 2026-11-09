/** Measure the scroll container, rather than the browser window or a guessed height. */
export function observeListViewport(element: HTMLElement, changed: (height: number) => void): () => void {
  let active = true;
  const measure = (): void => { if (active) changed(element.clientHeight); };
  const observer = typeof ResizeObserver === 'undefined' ? undefined : new ResizeObserver(measure);
  observer?.observe(element);
  window.addEventListener('resize', measure);
  // Refs run before insertion. Measure again once the mounted element has layout.
  queueMicrotask(measure);
  return () => { active = false; observer?.disconnect(); window.removeEventListener('resize', measure); };
}

/** A small rendered buffer, independent of the server's per-request page limit. */
export function viewportNeedsPage(top: number, height: number, rowHeight: number, loadedRows: number): boolean {
  return height > 0 && rowHeight > 0 && loadedRows < Math.ceil((Math.max(0, top) + height) / rowHeight) + 6;
}
