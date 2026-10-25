import type { JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { t } from '../i18n/index.ts';

/** Fallback for MIME types with no inline viewer — offers a download link only. */
export function UnsupportedViewer(props: ViewerProps): JSX.Element {
  return (
    <div class="mw-viewer__unsupported">
      <p>{t('common-viewer-no-preview')}</p>
      <p class="mw-viewer__unsupported-meta">
        {props.name} · {props.mime || t('common-viewer-unknown-type')}
      </p>
      <a class="btn btn--primary" href={props.blobUrl} download={props.name}>
        {t('common-viewer-download')}
      </a>
    </div>
  );
}

export default UnsupportedViewer;
