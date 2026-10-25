// useFieldErrors, useEtag and fieldAnchors (UX5, UX10, UX19).
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { createRoot, createSignal, Show } from 'solid-js';
import { pointerToFieldId, useFieldErrors } from './useFieldErrors.ts';
import { useEtag } from './useEtag.ts';
import {
  FIELD_HIGHLIGHT_ATTR,
  fieldAnchorAttrs,
  fieldAnchors,
  findFieldAnchor,
  focusField,
  focusFieldWhenReady,
  registerFieldAnchors,
} from './fieldAnchors.ts';
import { problem } from './uxTestkit.ts';

afterEach(() => vi.useRealTimers());

describe('pointerToFieldId', () => {
  it('maps JSON pointers to dotted field ids (RFC 6901 escapes)', () => {
    expect(pointerToFieldId('/smtp/port')).toBe('smtp.port');
    expect(pointerToFieldId('/globalValue')).toBe('globalValue');
    expect(pointerToFieldId('/a~1b/c~0d')).toBe('a/b.c~d');
    expect(pointerToFieldId('')).toBeUndefined();
    expect(pointerToFieldId('/')).toBeUndefined();
  });
});

describe('useFieldErrors', () => {
  it('a 422 with a pointer lands at that field, with the message for its code', () => {
    createRoot((dispose) => {
      const e = useFieldErrors({ known: () => ['smtp.port', 'smtp.host'] });
      expect(e.fromError(problem(422, { code: 'out_of_range', field: '/smtp/port' }))).toBe('smtp.port');
      expect(e.error('smtp.port')).toBe('The server refused this value because it is outside the allowed range.');
      expect(e.formError()).toBeNull();
      dispose();
    });
  });

  it("an unknown code shows the server's detail; a pointer to no shown field becomes the form error", () => {
    createRoot((dispose) => {
      const e = useFieldErrors({ known: () => ['a'] });
      e.fromError(problem(422, { code: 'weird', detail: 'must be lowercase', field: '/a' }));
      expect(e.error('a')).toBe('must be lowercase');
      expect(e.fromError(problem(422, { code: 'invalid', field: '/elsewhere' }))).toBeNull();
      expect(e.formError()).toBe('The server refused a value. Check the fields and try again.');
      dispose();
    });
  });

  it('a custom mapping places pointers the default cannot', () => {
    createRoot((dispose) => {
      const e = useFieldErrors({ fieldFor: (p) => (p === '/globalValue' ? 'signin.newUsers' : undefined) });
      expect(e.fromError(problem(422, { code: 'invalid', field: '/globalValue' }))).toBe('signin.newUsers');
      dispose();
    });
  });

  it('errors without a field (network, 403) are form errors', () => {
    createRoot((dispose) => {
      const e = useFieldErrors();
      expect(e.fromError(new TypeError('fetch failed'))).toBeNull();
      expect(e.formError()).toMatch(/server/i);
      expect(e.hasErrors()).toBe(true);
      e.clear();
      expect(e.hasErrors()).toBe(false);
      dispose();
    });
  });

  it('focusFirst focuses the first field in error in form order', () => {
    render(() => (
      <form>
        <div {...fieldAnchorAttrs('a')}>
          <input aria-label="A" />
        </div>
        <div {...fieldAnchorAttrs('b')}>
          <input aria-label="B" />
        </div>
      </form>
    ));
    createRoot((dispose) => {
      const e = useFieldErrors();
      e.replace({ b: 'x', a: 'y' });
      expect(e.focusFirst(['a', 'b'])).toBe(true);
      expect(document.activeElement).toBe(screen.getByLabelText('A'));
      dispose();
    });
  });
});

describe('useEtag', () => {
  it('sends the tag from the read as If-Match and keeps the tag the write returns', async () => {
    await createRoot(async (dispose) => {
      const tag = useEtag();
      tag.set('"v1"');
      const write = vi.fn(async (ifMatch: string | null) => ({ etag: '"v2"', sent: ifMatch }));
      const out = await tag.write(write);
      expect(out.sent).toBe('"v1"');
      expect(tag.etag()).toBe('"v2"');
      dispose();
    });
  });

  it('a 412 records who changed it; Keep mine repeats the write with the current tag', async () => {
    await createRoot(async (dispose) => {
      const tag = useEtag();
      tag.set('"v1"');
      const seen: (string | null)[] = [];
      const write = async (ifMatch: string | null) => {
        seen.push(ifMatch);
        if (ifMatch === '"v1"') throw problem(412, { code: 'stale', updatedBy: 'bob', updatedAt: '2026-10-25T10:00:00Z' }, '"v7"');
        return { etag: '"v8"' };
      };
      await expect(tag.write(write)).rejects.toMatchObject({ status: 412 });
      expect(tag.conflict()).toEqual({ updatedBy: 'bob', updatedAt: '2026-10-25T10:00:00Z' });
      await tag.keepMine();
      expect(seen).toEqual(['"v1"', '"v7"']);
      expect(tag.conflict()).toBeNull();
      expect(tag.etag()).toBe('"v8"');
      dispose();
    });
  });

  it('Keep mine reads the current tag when the 412 carried none', async () => {
    await createRoot(async (dispose) => {
      const refresh = vi.fn(async () => '"fresh"');
      const tag = useEtag({ refresh });
      tag.set('"old"');
      const seen: (string | null)[] = [];
      const write = async (ifMatch: string | null) => {
        seen.push(ifMatch);
        if (ifMatch === '"old"') throw problem(412, { code: 'stale' });
        return {};
      };
      await expect(tag.write(write)).rejects.toBeTruthy();
      await tag.keepMine();
      expect(refresh).toHaveBeenCalledTimes(1);
      expect(seen).toEqual(['"old"', '"fresh"']);
      dispose();
    });
  });

  it('other failures do not open a conflict; dismiss closes one', async () => {
    await createRoot(async (dispose) => {
      const tag = useEtag();
      await expect(tag.write(async () => Promise.reject(problem(422, { field: '/a' })))).rejects.toBeTruthy();
      expect(tag.conflict()).toBeNull();
      await expect(tag.write(async () => Promise.reject(problem(412, {})))).rejects.toBeTruthy();
      tag.dismiss();
      expect(tag.conflict()).toBeNull();
      await expect(tag.keepMine()).rejects.toThrow();
      dispose();
    });
  });
});

describe('fieldAnchors', () => {
  it('collects registered anchors with their section and catalog', () => {
    const remove = registerFieldAnchors({
      section: 'config',
      catalog: 'admin-config',
      anchors: [{ id: 'upstream.proxyTimeoutSecs', labelId: 'x-label', keywords: ['MW_PROXY_TIMEOUT'] }],
    });
    expect(findFieldAnchor('upstream.proxyTimeoutSecs')).toEqual({
      id: 'upstream.proxyTimeoutSecs',
      labelId: 'x-label',
      keywords: ['MW_PROXY_TIMEOUT'],
      section: 'config',
      catalog: 'admin-config',
    });
    remove();
    expect(fieldAnchors().some((a) => a.id === 'upstream.proxyTimeoutSecs')).toBe(false);
  });

  it('every anchor id is unique across the console', () => {
    const ids = fieldAnchors().map((a) => a.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('focusField focuses the control inside the holder and highlights the holder for a moment', () => {
    vi.useFakeTimers();
    render(() => (
      <div {...fieldAnchorAttrs('smtp.port')}>
        <label for="p">Port</label>
        <input id="p" />
      </div>
    ));
    expect(focusField('smtp.port')).toBe(true);
    expect(document.activeElement).toBe(screen.getByLabelText('Port'));
    const holder = screen.getByLabelText('Port').parentElement!;
    expect(holder).toHaveAttribute(FIELD_HIGHLIGHT_ATTR);
    vi.advanceTimersByTime(2500);
    expect(holder).not.toHaveAttribute(FIELD_HIGHLIGHT_ATTR);
    expect(focusField('nope')).toBe(false);
  });

  it('a disabled (pinned) field: the holder takes focus so its reason is read', () => {
    render(() => (
      <div {...fieldAnchorAttrs('pinned')} aria-label="Pinned field">
        <input disabled aria-label="Value" />
      </div>
    ));
    expect(focusField('pinned')).toBe(true);
    expect(document.activeElement).toBe(screen.getByLabelText('Pinned field'));
  });

  it('focusFieldWhenReady waits for a section that renders later', async () => {
    const [show, setShow] = createSignal(false);
    render(() => (
      <Show when={show()}>
        <div {...fieldAnchorAttrs('late')}>
          <input aria-label="Late" />
        </div>
      </Show>
    ));
    const done = focusFieldWhenReady('late', { timeoutMs: 2000 });
    setTimeout(() => setShow(true), 20);
    expect(await done).toBe(true);
    await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText('Late')));
    expect(await focusFieldWhenReady('never', { timeoutMs: 30 })).toBe(false);
  });
});
