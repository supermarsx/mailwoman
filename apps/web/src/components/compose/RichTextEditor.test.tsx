import { describe, it, expect, beforeAll } from 'vitest';
import { createSignal } from 'solid-js';
import { render, screen, waitFor, fireEvent } from '@solidjs/testing-library';
import { RichTextEditor, type RichTextApi } from './RichTextEditor.tsx';

beforeAll(()=>{Object.defineProperty(Range.prototype,'getClientRects',{configurable:true,value:()=>[]});Object.defineProperty(Range.prototype,'getBoundingClientRect',{configurable:true,value:()=>({left:0,right:0,top:0,bottom:0,width:0,height:0})});});

describe('RichTextEditor (W1)', () => {
  it('switches ribbon groups without changing the document',()=>{
    let html='';render(()=><RichTextEditor initialHtml="<p>Keep this</p>" ariaLabel="Body" onChange={value=>html=value}/>);
    expect(screen.getByLabelText('Font size')).toBeVisible();
    fireEvent.click(screen.getByRole('tab',{name:'Insert'}));
    expect(screen.getByTestId('rt-link')).toBeVisible();expect(screen.queryByLabelText('Font size')).toBeNull();
    expect(html).toBe('<p>Keep this</p>');
  });
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

it('edits real table structure, reverses it with history, and keeps display settings out of serialized HTML',()=>{
 let html='';render(()=><RichTextEditor initialHtml="<p>Keep</p>" ariaLabel="Body" onChange={value=>html=value}/>);
 fireEvent.click(screen.getByRole('tab',{name:'Insert'}));
 fireEvent.click(screen.getByRole('button',{name:'Insert table'}));
 expect(html).toContain('<table');expect((html.match(/<td/g)??[]).length).toBe(4);
 fireEvent.click(screen.getByRole('tab',{name:'Table'}));
 fireEvent.click(screen.getByRole('button',{name:'Row below'}));
 expect((html.match(/<td/g)??[]).length).toBe(6);
 fireEvent.click(screen.getByRole('button',{name:'Undo'}));
 expect((html.match(/<td/g)??[]).length).toBe(4);
 const before=html;fireEvent.change(screen.getByLabelText('Editor zoom'),{target:{value:'150'}});fireEvent.click(screen.getByRole('button',{name:'Paragraph marks'}));expect(html).toBe(before);
});
it('replaces literal text through the accessible controls and leaves unrelated marks intact',()=>{
 let html='';render(()=><RichTextEditor initialHtml="<p><strong>one</strong> two one</p>" ariaLabel="Body" onChange={value=>html=value}/>);
 fireEvent.click(screen.getByText('Find and replace'));
 fireEvent.input(screen.getByLabelText('Find text'),{target:{value:'one'}});fireEvent.input(screen.getByLabelText('Replace with'),{target:{value:'three'}});
 fireEvent.click(screen.getByRole('button',{name:'Replace all matches'}));expect(html).toBe('<p><strong>three</strong> two three</p>');
});
