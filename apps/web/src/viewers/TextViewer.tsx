import { createResource, Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { blobUrlToText, SANDBOX_TOKENS, textFrameDoc } from './sandbox.ts';

/** Plain-text attachment: content is escaped and inlined into an opaque-origin
 *  sandboxed frame — no external fetch, no script (plan §2.4). */
export function TextViewer(props: ViewerProps): JSX.Element {
  const [doc] = createResource(
    () => props.blobUrl,
    async (url) => textFrameDoc(await blobUrlToText(url)),
  );
  return (
    <Show when={doc()} fallback={<p class="mw-viewer__loading">Loading…</p>}>
      {(srcdoc) => (
        <iframe
          class="mw-viewer__frame mw-viewer__frame--text"
          title={props.name}
          sandbox={SANDBOX_TOKENS}
          srcdoc={srcdoc()}
        />
      )}
    </Show>
  );
}

export default TextViewer;
