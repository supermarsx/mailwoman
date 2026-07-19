import { describe, it, expect } from 'vitest';
import { createSignal } from 'solid-js';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { RichTextEditor, type RichTextApi } from './RichTextEditor.tsx';

describe('RichTextEditor (W1)', () => {
  it('mounts an editable region with the given accessible name', () => {
    render(() => (
      <RichTextEditor initialHtml="<p>hi</p>" ariaLabel="Body" onChange={() => undefined} />
    ));
    const editor = screen.getByTestId('compose-richtext');
    expect(editor.getAttribute('aria-label')).toBe('Body');
    expect(editor.getAttribute('contenteditable')).toBe('true');
  });

  it('emits serialized HTML + plain text on mount', () => {
    let html = '';
    let text = '';
    render(() => (
      <RichTextEditor
        initialHtml="<p>Hello <strong>world</strong></p>"
        ariaLabel="Body"
        onChange={(h, t) => {
          html = h;
          text = t;
        }}
      />
    ));
    expect(html).toContain('<strong>world</strong>');
    expect(text).toBe('Hello world');
  });

  it('setHtml replaces the document and re-emits', () => {
    let html = '';
    let api: RichTextApi | undefined;
    render(() => (
      <RichTextEditor
        initialHtml="<p>start</p>"
        ariaLabel="Body"
        onChange={(h) => (html = h)}
        onReady={(a) => (api = a)}
      />
    ));
    api?.setHtml('<p>replaced <em>text</em></p>');
    expect(html).toContain('replaced');
    expect(html).toContain('<em>text</em>');
  });

  it('appendHtml adds blocks at the end', () => {
    let text = '';
    let api: RichTextApi | undefined;
    render(() => (
      <RichTextEditor
        initialHtml="<p>first</p>"
        ariaLabel="Body"
        onChange={(_h, t) => (text = t)}
        onReady={(a) => (api = a)}
      />
    ));
    api?.appendHtml('<p>second</p>');
    expect(text).toBe('first\n\nsecond');
  });

  it('reconciles an external plain-text change into the document', async () => {
    const [ext, setExt] = createSignal('one');
    let text = '';
    render(() => (
      <RichTextEditor
        initialHtml="<p>one</p>"
        ariaLabel="Body"
        externalText={ext}
        onChange={(_h, t) => (text = t)}
      />
    ));
    setExt('two\nthree');
    await waitFor(() => expect(text).toBe('two\nthree'));
  });
});
