import { createResource, Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { blobUrlToDataUrl, mediaFrameDoc, SANDBOX_TOKENS } from './sandbox.ts';

/** Native <video controls> inside an opaque-origin sandboxed frame — NO transcode
 *  in V2 (plan §0.10 / §1.7); the browser plays what it natively supports. */
export function VideoViewer(props: ViewerProps): JSX.Element {
  const [doc] = createResource(
    () => props.blobUrl,
    async (url) => mediaFrameDoc('video', await blobUrlToDataUrl(url)),
  );
  return (
    <Show when={doc()} fallback={<p class="mw-viewer__loading">Loading video…</p>}>
      {(srcdoc) => (
        <iframe
          class="mw-viewer__frame mw-viewer__frame--video"
          title={props.name}
          sandbox={SANDBOX_TOKENS}
          srcdoc={srcdoc()}
        />
      )}
    </Show>
  );
}

export default VideoViewer;
