import {previewToolPlacement} from './filePreviewPreferences.ts';
import {previewKindEnabled} from './filePreviewPreferences.ts';
// `<AttachmentViewer>` — dispatches by MIME to a per-type viewer (plan §2.4).
//
// EVERY concrete viewer is loaded via `lazy(() => import(...))`, so each is a
// separate chunk. Critically the full PDF viewer's self-hosted frame assets
// are only ever pulled in on demand — pdfjs never lands in the
// login→inbox entry chunk (plan §1.7, §23 bundle gate; asserted in the specs).

import { ErrorBoundary, lazy, Match, Show, Suspense, Switch, type JSX } from 'solid-js';
import { viewerKindFor, type ViewerProps } from '../contracts/viewer.ts';
import { t } from '../i18n/index.ts';
import { mailFeature } from '../state/mailFeatures.ts';
import { loadCatalog } from '../i18n/catalog.ts';
import { ErrorReference } from '../components/ErrorReference.tsx';
import { reportBoundaryError } from '../observability/clientErrors.ts';
import './text-viewer.css';

const MessageAttachmentViewer = lazy(() => import('./MessageAttachmentViewer.tsx'));
const CalendarAttachmentViewer = lazy(() => import('./CalendarAttachmentViewer.tsx'));
const ImageViewer = lazy(() => import('./ImageViewer.tsx'));
const PdfViewer = lazy(() => import('./PdfViewer.tsx'));
const HtmlViewer = lazy(() => import('./HtmlViewer.tsx'));
const TextViewer = lazy(() => import('./TextViewer.tsx'));
const AudioViewer = lazy(() => import('./AudioViewer.tsx'));
const VideoViewer = lazy(() => import('./VideoViewer.tsx'));
const UnsupportedViewer = lazy(() => import('./UnsupportedViewer.tsx'));

export function AttachmentViewer(props: ViewerProps): JSX.Element {
  void loadCatalog('text-viewer'); void loadCatalog('file-previews');
  const kind = (): ReturnType<typeof viewerKindFor> => previewKindEnabled(viewerKindFor(props.mime,props.name))?viewerKindFor(props.mime,props.name):'unsupported';
  return (
    <div class="mw-viewer" data-viewer-kind={kind()}>
      {/* A viewer that throws stays in its own panel: Retry re-renders only the preview. */}
      <ErrorBoundary fallback={(error: unknown, reset: () => void) => {
        const linked = error instanceof Error && error.name === 'LinkedContextUnsupportedError';
        const report = linked ? undefined : reportBoundaryError(error, `Reader/AttachmentViewer/${kind()}`);
        if (!linked) console.warn('[viewer] preview failed', { type: props.mime, kind: kind(), error: error instanceof Error ? error.name : typeof error });
        return (
          <div class="mw-text__state" role="alert" data-viewer-failed="">
            <p><strong>{t(linked ? 'text-viewer-linked-unavailable' : 'text-viewer-failed')}</strong></p>
            <Show when={!linked}><p>{t('text-viewer-failed-help')}</p></Show>
            <div class="mw-text__actions">
              <Show when={!linked}><button type="button" class="btn" onClick={reset}>{t('text-viewer-retry')}</button></Show>
              <Show when={mailFeature('allowDownload')&&previewToolPlacement('download')!=='hidden'}><a class="btn" href={props.blobUrl} download={props.name.replace(/[\\/]/g, '_')}>{t('text-viewer-download')}</a></Show>
            </div>
            <Show when={report}><ErrorReference report={report!} /></Show>
          </div>
        );
      }}>
      <Suspense fallback={<p class="mw-viewer__loading">{t('common-viewer-loading')}</p>}>
        <Switch fallback={<UnsupportedViewer {...props} />}>
          <Match when={kind() === 'message'}><MessageAttachmentViewer {...props} /></Match>
          <Match when={kind() === 'calendar'}><CalendarAttachmentViewer {...props} /></Match>
          <Match when={kind() === 'image'}>
            <ImageViewer {...props} />
          </Match>
          <Match when={kind() === 'pdf'}>
            <PdfViewer {...props} />
          </Match>
          <Match when={kind() === 'html'}><HtmlViewer {...props}/></Match>
          <Match when={kind() === 'text'||kind()==='code'}>
            <TextViewer {...props} />
          </Match>
          <Match when={kind() === 'audio'}>
            <AudioViewer {...props} />
          </Match>
          <Match when={kind() === 'video'}>
            <VideoViewer {...props} />
          </Match>
        </Switch>
      </Suspense>
      </ErrorBoundary>
    </div>
  );
}

export default AttachmentViewer;
