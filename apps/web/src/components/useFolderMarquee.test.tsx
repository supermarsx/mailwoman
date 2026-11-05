// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@solidjs/testing-library';
import { createSignal, For } from 'solid-js';
import { useFolderMarquee } from './useFolderMarquee.tsx';
import { clearSelectedFolders, selectedFolderIds, setSelectedFolders } from './folderSelection.ts';

class TestPointerEvent extends MouseEvent {
  readonly pointerId: number; readonly pointerType: string;
  constructor(type: string, init: PointerEventInit = {}) { super(type, { bubbles: true, cancelable: true, ...init }); this.pointerId = init.pointerId ?? 1; this.pointerType = init.pointerType ?? 'mouse'; }
}
const rect = (top: number, height: number, width = 200): DOMRect => ({ left: 0, top, width, height, right: width, bottom: top + height, x: 0, y: top, toJSON: () => ({}) }) as DOMRect;
let frames: FrameRequestCallback[] = [];

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
  vi.stubGlobal('PointerEvent', TestPointerEvent);
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { frames.push(cb); return frames.length; });
  vi.stubGlobal('cancelAnimationFrame', () => { frames = []; });
});
afterEach(() => { cleanup(); clearSelectedFolders(); vi.useRealTimers(); vi.unstubAllGlobals(); frames = []; });

function Harness(props: { ids: () => string[] }) {
  const marquee = useFolderMarquee(props.ids);
  return <div data-testid="panel" ref={marquee.ref}>
    {marquee.liveRegion()}
    <div data-folder-region>
      <For each={props.ids()}>{id => <button type="button" data-marquee-item="" data-folder-id={id} data-selected={selectedFolderIds().has(id) ? 'true' : undefined}>{id}</button>}</For>
    </div>
    <div data-testid="gap" />
  </div>;
}
function layout(): void {
  const panel = screen.getByTestId('panel');
  panel.getBoundingClientRect = () => rect(0, 300);
  Object.defineProperty(panel, 'clientWidth', { value: 200, configurable: true });
  Object.defineProperty(panel, 'clientHeight', { value: 300, configurable: true });
  panel.querySelectorAll<HTMLElement>('[data-folder-id]').forEach((el, index) => { el.getBoundingClientRect = () => rect(index * 30, 28); });
}
const fire = (target: EventTarget, type: string, init: PointerEventInit = {}): void => { target.dispatchEvent(new TestPointerEvent(type, init)); };
const flush = (): void => { const run = frames; frames = []; run.forEach(cb => cb(0)); };

describe('useFolderMarquee', () => {
  it('drag over rows marks them selected and announces the count once', () => {
    render(() => <Harness ids={() => ['inbox', 'sent', 'work', 'home']} />);
    layout();
    fire(screen.getByTestId('gap'), 'pointerdown', { clientX: 100, clientY: 250 });
    fire(document, 'pointermove', { clientX: 100, clientY: 40 }); flush();
    fire(document, 'pointerup', { clientX: 100, clientY: 40 });
    expect([...selectedFolderIds()].sort()).toEqual(['home', 'sent', 'work']);
    expect(screen.getByRole('button', { name: 'sent' })).toHaveAttribute('data-selected', 'true');
    expect(screen.getByRole('button', { name: 'inbox' })).not.toHaveAttribute('data-selected');
    expect(screen.getByTestId('folder-select-status').textContent).toBe('');
    vi.advanceTimersByTime(400);
    expect(screen.getByTestId('folder-select-status').textContent).toMatch(/3 folders selected/);
    expect(screen.getByTestId('folder-select-status')).toHaveAttribute('aria-live', 'polite');
  });
  it('a click on empty space clears the selection and announces it', () => {
    render(() => <Harness ids={() => ['inbox', 'sent']} />);
    layout(); setSelectedFolders(new Set(['inbox', 'sent']));
    vi.advanceTimersByTime(400);
    fire(screen.getByTestId('gap'), 'pointerdown', { clientX: 100, clientY: 250 });
    fire(document, 'pointerup', { clientX: 100, clientY: 250 });
    expect(selectedFolderIds().size).toBe(0);
    vi.advanceTimersByTime(400);
    expect(screen.getByTestId('folder-select-status').textContent).toMatch(/cleared/i);
  });
  it('a press on a folder row does not start a marquee', () => {
    render(() => <Harness ids={() => ['inbox', 'sent']} />);
    layout(); setSelectedFolders(new Set(['sent']));
    fire(screen.getByRole('button', { name: 'inbox' }), 'pointerdown', { clientX: 10, clientY: 10 });
    fire(document, 'pointermove', { clientX: 100, clientY: 100 }); flush();
    fire(document, 'pointerup');
    expect([...selectedFolderIds()]).toEqual(['sent']);
  });
  it('folders that disappear (collapsed away) drop out of the selection', () => {
    const [ids, setIds] = createSignal(['inbox', 'work', 'q3']);
    render(() => <Harness ids={ids} />);
    setSelectedFolders(new Set(['work', 'q3']));
    setIds(['inbox', 'work']);
    expect([...selectedFolderIds()]).toEqual(['work']);
  });
});
