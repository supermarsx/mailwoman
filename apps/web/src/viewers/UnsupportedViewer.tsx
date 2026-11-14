import { Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import {ViewerToolbar,downloadPreview} from './ViewerToolbar.tsx';
import { t } from '../i18n/index.ts';
import { mailFeature } from '../state/mailFeatures.ts';

/** Fallback for MIME types with no inline viewer — offers a download link only. */
export function UnsupportedViewer(props: ViewerProps): JSX.Element {
  return (
    <div class="mw-viewer__unsupported">
      <p>{t('common-viewer-no-preview')}</p>
      <p class="mw-viewer__unsupported-meta">
        {props.name} · {props.mime || t('common-viewer-unknown-type')}
      </p>
      <Show when={mailFeature('allowDownload')}>
        <ViewerToolbar scope={props.blobUrl} actions={[{id:'download',label:t('common-viewer-download'),icon:'download',run:()=>downloadPreview(props.blobUrl,props.name)}]}/>
      </Show>
    </div>
  );
}

export default UnsupportedViewer;
