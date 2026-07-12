import { createResource, Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { blobUrlToDataUrl, mediaFrameDoc, SANDBOX_TOKENS } from './sandbox.ts';

/** Native <audio controls> inside an opaque-origin sandboxed frame (no transcode). */
export function AudioViewer(props: ViewerProps): JSX.Element {
  const [doc] = createResource(
    () => props.blobUrl,
    async (url) => mediaFrameDoc('audio', await blobUrlToDataUrl(url)),
  );
  return (
    <Show when={doc()} fallback={<p class="mw-viewer__loading">Loading audio…</p>}>
      {(srcdoc) => (
        <iframe
          class="mw-viewer__frame mw-viewer__frame--audio"
          title={props.name}
          sandbox={SANDBOX_TOKENS}
          srcdoc={srcdoc()}
        />
      )}
    </Show>
  );
}

export default AudioViewer;
