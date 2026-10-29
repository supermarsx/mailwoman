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
    expect(screen.getByTestId('rt-link')).toBeVisible();expect(screen.getByLabelText('Font size')).not.toBeVisible();
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
 fireEvent.click(screen.getByRole('tab',{name:'Format text'}));
 const before=html;fireEvent.change(screen.getByLabelText('Editor zoom'),{target:{value:'150'}});fireEvent.click(screen.getByRole('button',{name:'Paragraph marks'}));expect(html).toBe(before);
});
it('replaces literal text through the accessible controls and leaves unrelated marks intact',()=>{
 let html='';render(()=><RichTextEditor initialHtml="<p><strong>one</strong> two one</p>" ariaLabel="Body" onChange={value=>html=value}/>);
 fireEvent.click(screen.getByText('Find and replace'));
 fireEvent.input(screen.getByLabelText('Find text'),{target:{value:'one'}});fireEvent.input(screen.getByLabelText('Replace with'),{target:{value:'three'}});
 fireEvent.click(screen.getByRole('button',{name:'Replace all matches'}));expect(html).toBe('<p><strong>three</strong> two three</p>');
});

it('mounts rich controls in the supplied ribbon and keeps one editable document',()=>{
 const ribbon=document.createElement('div');document.body.append(ribbon);
 const result=render(()=><RichTextEditor toolbarMount={ribbon} initialHtml="<p>Retained</p>" ariaLabel="Body" onChange={()=>undefined}/>);
 const editor=screen.getByTestId('compose-richtext');
 expect(ribbon.querySelector('[data-testid="rt-bold"]')).not.toBeNull();
 expect(result.container.querySelector('.compose-rt__chrome')).toBeNull();
 fireEvent.click(screen.getByRole('tab',{name:'Insert'}));
 expect(screen.getByTestId('compose-richtext')).toBe(editor);
 expect(editor).toHaveTextContent('Retained');
 result.unmount();expect(ribbon.children).toHaveLength(0);ribbon.remove();
});

// t39 L12: the View group's Dark background button changes the surface only.
describe('RichTextEditor dark background (t39 L12)', () => {
  it('flips the surface, shows unreadable colours readably, and leaves the HTML unchanged', async () => {
    const painted = (el: Element): string => (el as HTMLElement).dataset['tone'] === 'dark' ? 'rgb(30, 31, 34)' : 'rgb(255, 255, 255)';
    const real = window.getComputedStyle.bind(window);
    const spy = (el: Element, pseudo?: string | null): CSSStyleDeclaration => {
      const style = real(el, pseudo);
      if (!el.classList.contains('compose-rt__surface')) return style;
      return new Proxy(style, { get: (target, key) => key === 'backgroundColor' ? painted(el) : Reflect.get(target, key) });
    };
    Object.defineProperty(window, 'getComputedStyle', { configurable: true, value: spy });
    Object.defineProperty(globalThis, 'getComputedStyle', { configurable: true, value: spy });
    try {
      const htmls: string[] = [];
      const initial = '<p><span style="color: #000000">ink</span> and <span style="color: #ffffff">chalk</span></p>';
      render(() => <RichTextEditor initialHtml={initial} ariaLabel="Body" onChange={value => htmls.push(value)} />);
      const button = screen.getByTestId('rt-dark-background');
      const surface = document.querySelector('.compose-rt__surface') as HTMLElement;
      expect(button).toHaveAccessibleName('Dark background');
      expect(button).toHaveAttribute('aria-pressed', 'false');
      expect(surface.dataset['surface']).toBe('theme');
      const shown = (): string[] => [...surface.querySelectorAll('[data-mw-view-color]')].map(el => el.textContent ?? '');
      // On the light surface the white text is the unreadable one.
      expect(shown()).toEqual(['chalk']);
      const sent = htmls.at(-1);
      fireEvent.click(button);
      await waitFor(() => expect(button).toHaveAttribute('aria-pressed', 'true'));
      expect(surface.dataset['surface']).toBe('dark');
      await waitFor(() => expect(shown()).toEqual(['ink']));
      fireEvent.click(button);
      await waitFor(() => expect(surface.dataset['surface']).toBe('light'));
      await waitFor(() => expect(shown()).toEqual(['chalk']));
      // No change to the document was emitted: what is sent is what was there.
      expect(htmls.at(-1)).toBe(sent);
      expect(sent).toContain('chalk</span>');
      expect(sent).toMatch(/color: (#ffffff|rgb\(255, 255, 255\))/);
      expect(sent).not.toContain('data-mw-view-color');
    } finally {
      Object.defineProperty(window, 'getComputedStyle', { configurable: true, value: real });
      Object.defineProperty(globalThis, 'getComputedStyle', { configurable: true, value: real });
    }
  });
});
