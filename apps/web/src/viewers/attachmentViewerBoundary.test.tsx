// A viewer that throws must be contained by the AttachmentViewer's own boundary:
// the section-level boundary never sees it, and Retry re-renders the preview.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@solidjs/testing-library';
import { ErrorBoundary } from 'solid-js';

let failures = 1;
vi.mock('./TextViewer.tsx', () => ({
  default: () => {
    if (failures-- > 0) throw new RangeError('The encoding label provided is invalid');
    return <p data-testid="recovered">recovered</p>;
  },
}));

import { AttachmentViewer } from './AttachmentViewer.tsx';

afterEach(() => { failures = 1; vi.restoreAllMocks(); });

describe('AttachmentViewer boundary', () => {
  it('contains a throwing viewer inline and Retry re-runs it', async () => {
    vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const outer: unknown[] = [];
    const view = render(() => (
      <ErrorBoundary fallback={(e) => { outer.push(e); return <p>section boundary</p>; }}>
        <AttachmentViewer part={{ partId: null, blobId: 'b', size: 1, type: 'text/plain' }} blobUrl="blob:x" mime="text/plain" name="a.txt" />
      </ErrorBoundary>
    ));
    await waitFor(() => expect(view.container.querySelector('[data-viewer-failed]')).not.toBeNull());
    expect(outer).toEqual([]);
    const retry = view.container.querySelector('[data-viewer-failed] button') as HTMLButtonElement;
    fireEvent.click(retry);
    await waitFor(() => expect(view.getByTestId('recovered')).toBeTruthy());
    expect(outer).toEqual([]);
  });
});
