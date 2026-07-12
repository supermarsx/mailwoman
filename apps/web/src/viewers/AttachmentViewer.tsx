// `<AttachmentViewer>` — dispatches by MIME to a per-type viewer (plan §2.4).
//
// EVERY concrete viewer is loaded via `lazy(() => import(...))`, so each is a
// separate chunk. Critically the PDF viewer (which statically imports the ~1 MB
// pdfjs) is only ever pulled in on demand — pdfjs never lands in the
// login→inbox entry chunk (plan §1.7, §23 bundle gate; asserted in the specs).

import { lazy, Match, Suspense, Switch, type JSX } from 'solid-js';
import { viewerKindFor, type ViewerProps } from '../contracts/viewer.ts';

const ImageViewer = lazy(() => import('./ImageViewer.tsx'));
const PdfViewer = lazy(() => import('./PdfViewer.tsx'));
const TextViewer = lazy(() => import('./TextViewer.tsx'));
const AudioViewer = lazy(() => import('./AudioViewer.tsx'));
const VideoViewer = lazy(() => import('./VideoViewer.tsx'));
const UnsupportedViewer = lazy(() => import('./UnsupportedViewer.tsx'));

export function AttachmentViewer(props: ViewerProps): JSX.Element {
  const kind = (): ReturnType<typeof viewerKindFor> => viewerKindFor(props.mime);
  return (
    <div class="mw-viewer" data-viewer-kind={kind()}>
      <Suspense fallback={<p class="mw-viewer__loading">Loading viewer…</p>}>
        <Switch fallback={<UnsupportedViewer {...props} />}>
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
    </div>
  );
}

export default AttachmentViewer;
