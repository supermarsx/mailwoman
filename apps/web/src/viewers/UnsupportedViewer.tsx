import type { JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';

/** Fallback for MIME types with no inline viewer — offers a download link only. */
export function UnsupportedViewer(props: ViewerProps): JSX.Element {
  return (
    <div class="mw-viewer__unsupported">
      <p>No inline preview for this file type.</p>
      <p class="mw-viewer__unsupported-meta">
        {props.name} · {props.mime || 'unknown type'}
      </p>
      <a class="btn btn--primary" href={props.blobUrl} download={props.name}>
        Download
      </a>
    </div>
  );
}

export default UnsupportedViewer;
