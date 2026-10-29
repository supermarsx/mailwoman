// The slot the message body frame sits in (t79). It applies the reader zoom
// (readerZoom.ts) from outside the frame: a level change only updates one custom
// property here, so the message document is never reloaded, its remote images
// are not fetched again, and its scroll position stays where it was. See
// readerZoom.css.ts for the geometry. The frame document itself carries no zoom,
// so printing it (printMessage.ts builds its own copy) stays at 100%.

import { createEffect, type JSX } from 'solid-js';
import { readerZoom } from './readerZoom.ts';
import * as css from './readerZoom.css.ts';

export function ReaderZoomBox(props: { children: JSX.Element }): JSX.Element {
  let box: HTMLDivElement | undefined;
  createEffect(() => {
    const scale = readerZoom() / 100;
    // CSSOM, not a style attribute: the app CSP refuses inline styles.
    if (scale === 1) box?.style.removeProperty(css.zoomVar);
    else box?.style.setProperty(css.zoomVar, String(scale));
  });
  return <div ref={box} class={css.box} data-testid="reader-zoom-box" data-reader-zoom={readerZoom()}>{props.children}</div>;
}
