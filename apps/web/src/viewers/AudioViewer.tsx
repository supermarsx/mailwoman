import {ViewerToolbar} from './ViewerToolbar.tsx';
import {previewToolPlacement} from './filePreviewPreferences.ts';
import { createResource,createSignal, Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { blobUrlToDataUrl, mediaFrameDoc, SANDBOX_TOKENS } from './sandbox.ts';
import { t } from '../i18n/index.ts';

/** Native <audio controls> inside an opaque-origin sandboxed frame (no transcode). */
export function AudioViewer(props: ViewerProps): JSX.Element {
  const [controls,setControls]=createSignal(false);
  const showControls=()=>previewToolPlacement('media.controls')==='toolbar'||previewToolPlacement('media.controls')==='menu'&&controls();
  const [doc] = createResource(
    () => ({url:props.blobUrl,controls:showControls()}),
    async ({url,controls}) => mediaFrameDoc('audio', await blobUrlToDataUrl(url),controls),
  );
  return (
    <><div class="mw-text__bar"><ViewerToolbar scope={props.blobUrl} actions={[{id:'media.controls',label:t('preview-tool-media-controls'),icon:'settings',pressed:showControls(),render:()=>null,run:()=>{setControls(value=>!value);}}]}/></div><Show when={doc()} fallback={<p class="mw-viewer__loading">{t('common-viewer-loading-audio')}</p>}>
      {(srcdoc) => (
        <iframe
          class="mw-viewer__frame mw-viewer__frame--audio"
          title={props.name}
          sandbox={SANDBOX_TOKENS}
          srcdoc={srcdoc()}
        />
      )}
    </Show></>
  );
}

export default AudioViewer;
