import { createResource, Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { blobUrlToDataUrl, mediaFrameDoc, SANDBOX_TOKENS } from './sandbox.ts';

/** Native <img> inside an opaque-origin sandboxed frame (plan §2.4). */
export function ImageViewer(props: ViewerProps): JSX.Element {
  const [doc] = createResource(
    () => props.blobUrl,
    async (url) => mediaFrameDoc('image', await blobUrlToDataUrl(url)),
  );
  return (
    <Show when={doc()} fallback={<p class="mw-viewer__loading">Loading image…</p>}>
      {(srcdoc) => (
        <iframe
          class="mw-viewer__frame mw-viewer__frame--image"
          title={props.name}
          sandbox={SANDBOX_TOKENS}
          srcdoc={srcdoc()}
        />
      )}
    </Show>
  );
}

export default ImageViewer;
