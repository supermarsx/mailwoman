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
const TextViewer = lazy(() => import('./TextViewer.tsx'));
const AudioViewer = lazy(() => import('./AudioViewer.tsx'));
const VideoViewer = lazy(() => import('./VideoViewer.tsx'));
const UnsupportedViewer = lazy(() => import('./UnsupportedViewer.tsx'));

export function AttachmentViewer(props: ViewerProps): JSX.Element {
  void loadCatalog('text-viewer');
  const kind = (): ReturnType<typeof viewerKindFor> => viewerKindFor(props.mime, props.name);
  return (
    <div class="mw-viewer" data-viewer-kind={kind()}>
      {/* A viewer that throws stays in its own panel: Retry re-renders only the preview. */}
      <ErrorBoundary fallback={(error: unknown, reset: () => void) => {
        const report = reportBoundaryError(error, `Reader/AttachmentViewer/${kind()}`);
        console.warn('[viewer] preview failed', { type: props.mime, kind: kind(), error: error instanceof Error ? error.name : typeof error });
        return (
          <div class="mw-text__state" role="alert" data-viewer-failed="">
            <p><strong>{t('text-viewer-failed')}</strong></p>
            <p>{t('text-viewer-failed-help')}</p>
            <div class="mw-text__actions">
              <button type="button" class="btn" onClick={reset}>{t('text-viewer-retry')}</button>
              <Show when={mailFeature('allowDownload')}><a class="btn" href={props.blobUrl} download={props.name.replace(/[\\/]/g, '_')}>{t('text-viewer-download')}</a></Show>
            </div>
            <ErrorReference report={report} />
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
          <Match when={kind() === 'text'}>
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
