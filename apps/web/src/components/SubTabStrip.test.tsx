import {setAdminLeetspeak} from '../i18n/presentation.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import { toasts, resetToasts } from './toast/index.ts';
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { SubTabStrip } from './SubTabStrip.tsx';
import { createSignal } from 'solid-js';
import type { BarMode, BarModeHandle } from './bars/index.ts';
import { RealtimeContext } from '../realtime/context.ts';
import { createRealtimeController, type RealtimeController } from '../realtime/controller.ts';

function renderStrip(controller: RealtimeController, bar?: BarModeHandle) {
  return render(() => (
    <RealtimeContext.Provider value={controller}>
      <SubTabStrip {...(bar ? { bar } : {})} />
    </RealtimeContext.Provider>
  ));
}

function fakeBar(initial: BarMode): BarModeHandle & { calls: BarMode[] } {
  const [mode, setMode] = createSignal<BarMode>(initial);
  const calls: BarMode[] = [];
  return {
    calls, mode, locked: () => false, saving: () => false, error: () => '',
    setMode: (m) => { calls.push(m); setMode(m); return Promise.resolve(true); },
  };
}

describe('SubTabStrip as the workspaceTabs bar (t38)', () => {
  const barRoot = (): HTMLElement => document.querySelector('[data-bar="workspaceTabs"]') as HTMLElement;
  it('transforms only displayed tab labels and preserves title/identity through activation',()=>{
    const controller=createRealtimeController({subTabs:{openWindow:vi.fn()}});
    controller.subTabs.open({kind:'messages',title:'Inbox Messages'});
    const original=controller.subTabs.tabs()[0]!;
    setAdminLeetspeak(true);
    try{
      renderStrip(controller);
      fireEvent.click(screen.getByRole('tab',{name:'1n80x M3554935'}));
      expect(controller.subTabs.activeId()).toBe(original.id);
      expect(controller.subTabs.tabs()[0]!.title).toBe('Inbox Messages');
      setAdminLeetspeak(false);
      expect(screen.getByRole('tab',{name:'Inbox Messages'})).toBeInTheDocument();
    }finally{setAdminLeetspeak(null);}
  });

  it('pinned: the strip is shown with a pin and no handle', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    const bar = fakeBar('pinned');
    renderStrip(controller, bar);
    expect(barRoot()).toHaveAttribute('data-mode', 'pinned');
    expect(barRoot()).toHaveAttribute('data-reveal', 'overlay');
    expect(document.querySelector('[data-bar-handle]')).toBeNull();
    fireEvent.click(document.querySelector('[data-bar-pin="workspaceTabs"]')!);
    expect(bar.calls).toEqual(['auto']);
  });
  it('auto: a slim handle names the active tab; focus opens the strip and Esc closes it', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    controller.subTabs.open({ kind: 'module', title: 'Contacts' });
    renderStrip(controller, fakeBar('auto'));
    const handle = document.querySelector('[data-bar-handle="workspaceTabs"]') as HTMLButtonElement;
    expect(handle).toHaveTextContent('Contacts');
    expect(handle).toHaveAttribute('aria-expanded', 'false');
    expect(barRoot()).toHaveAttribute('data-expanded', 'false');
    expect(document.getElementById('mw-bar-workspaceTabs')).toHaveAttribute('inert');
    handle.focus();
    expect(barRoot()).toHaveAttribute('data-expanded', 'true');
    expect(handle).toHaveAttribute('aria-expanded', 'true');
    const tab = screen.getByRole('tab', { name: 'Contacts' });
    tab.focus();
    fireEvent.keyDown(tab, { key: 'Escape' });
    expect(barRoot()).toHaveAttribute('data-expanded', 'false');
    expect(document.activeElement).toBe(handle);
  });
  it('a vertical wheel scrolls the overflowing strip sideways', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    renderStrip(controller, fakeBar('pinned'));
    const strip = document.querySelector('.subtab-strip') as HTMLElement;
    Object.defineProperty(strip, 'scrollWidth', { configurable: true, value: 900 });
    Object.defineProperty(strip, 'clientWidth', { configurable: true, value: 300 });
    const wheel = new WheelEvent('wheel', { deltaY: 100, bubbles: true, cancelable: true });
    strip.dispatchEvent(wheel);
    expect(wheel.defaultPrevented).toBe(true);
    expect(strip.scrollLeft).toBe(100);
  });
});

describe('SubTabStrip', () => {
  it('enables native pointer dragging with an explicit enumerated attribute',()=>{
    const controller=createRealtimeController();controller.subTabs.open({kind:'module',title:'Contacts'});renderStrip(controller);
    const tab=screen.getByRole('tab',{name:'Contacts'});expect(tab.getAttribute('draggable')).toBe('true');expect(tab.draggable).toBe(true);
  });
  it('accepts only its own current-account drag token',()=>{
    activateAccountPreferences('tab-owner');
    const controller=createRealtimeController();const a=controller.subTabs.open({kind:'messages',title:'Inbox'}),b=controller.subTabs.open({kind:'module',title:'Contacts'});renderStrip(controller);
    const values=new Map<string,string>();const transfer={types:[] as string[],effectAllowed:'',dropEffect:'',setData:(type:string,value:string)=>{values.set(type,value);transfer.types=[type];},getData:(type:string)=>values.get(type)??''};
    fireEvent.dragStart(screen.getByRole('tab',{name:'Contacts'}),{dataTransfer:transfer});
    fireEvent.drop(screen.getByRole('tab',{name:'Inbox'}),{dataTransfer:transfer});expect(controller.subTabs.tabs().map(tab=>tab.id)).toEqual([b,a]);
    fireEvent.dragStart(screen.getByRole('tab',{name:'Contacts'}),{dataTransfer:transfer});suspendAccountPreferences();activateAccountPreferences('different');
    fireEvent.drop(screen.getByRole('tab',{name:'Inbox'}),{dataTransfer:transfer});expect(controller.subTabs.tabs().map(tab=>tab.id)).toEqual([b,a]);suspendAccountPreferences();
  });
  it('moves the active tab with Ctrl+Shift arrows without changing its identity',()=>{
    const controller=createRealtimeController();
    const a=controller.subTabs.open({kind:'messages',title:'Inbox'});
    const b=controller.subTabs.open({kind:'module',title:'Contacts'});
    renderStrip(controller);
    fireEvent.keyDown(screen.getByRole('tab',{name:'Contacts'}),{key:'ArrowLeft',ctrlKey:true,shiftKey:true});
    expect(controller.subTabs.tabs().map(tab=>tab.id)).toEqual([b,a]);
    expect(controller.subTabs.activeId()).toBe(b);
  });
  it('closing the active tab with its close button moves focus to the tab that is active next', async () => {
    // t33-e10: the close button is removed with its tab, so focus fell to <body>.
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    controller.subTabs.open({ kind: 'module', title: 'Plans' });
    renderStrip(controller);
    const closeButton = screen.getByRole('button', { name: 'Close Plans' });
    closeButton.focus();
    fireEvent.click(closeButton);
    await Promise.resolve();
    await Promise.resolve();
    expect(screen.queryByRole('tab', { name: 'Plans' })).toBeNull();
    expect(document.activeElement).toBe(screen.getByRole('tab', { name: 'Inbox' }));
  });
  it('renders nothing while there are no tabs', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    renderStrip(controller);
    expect(screen.queryByRole('tablist')).toBeNull();
  });

  it('renders a tab per open surface and marks the active one selected', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    const b = controller.subTabs.open({ kind: 'composer', title: 'Draft' });
    renderStrip(controller);
    expect(screen.getAllByRole('tab')).toHaveLength(2);
    // The most recently opened tab is active.
    expect(screen.getByRole('tab', { selected: true })).toHaveTextContent('Draft');
    expect(controller.subTabs.activeId()).toBe(b);
  });

  it('clicking a tab activates it', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    const a = controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    controller.subTabs.open({ kind: 'composer', title: 'Draft' });
    renderStrip(controller);
    fireEvent.click(screen.getByRole('tab', { name: 'Inbox' }));
    expect(controller.subTabs.activeId()).toBe(a);
    expect(screen.getByRole('tab', { selected: true })).toHaveTextContent('Inbox');
  });

  it('the close button removes the tab', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    renderStrip(controller);
    fireEvent.click(screen.getByRole('button', { name: 'Close Inbox' }));
    expect(screen.queryByRole('tab')).toBeNull();
  });

  it('the context menu stars without an inline toggle and names the tab in feedback', async () => {
    resetToasts();
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    renderStrip(controller);
    fireEvent.contextMenu(screen.getByRole('tab',{name:'Inbox'}));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Star' }));
    expect(controller.subTabs.tabs()[0]!.pinned).toBe(true);
    // The label flips to the unpin affordance.
    expect(screen.queryByRole('button',{name:'Unpin Inbox'})).toBeNull();
    expect(screen.getByRole('img',{name:'Starred tab'})).toBeInTheDocument();
    fireEvent.keyDown(screen.getByRole('tab',{name:'Inbox'}),{key:'F10',shiftKey:true});
    expect(screen.getByRole('menuitem',{name:'Unstar'})).toBeInTheDocument();
    expect(toasts()[0]?.message).toContain('Pinned tab');
    expect(toasts()[0]?.message).toContain('Inbox');
  });

  it('offers detach only with a real retained-pane handler and keeps the tab owned', () => {
    const controller = createRealtimeController();
    controller.subTabs.open({ id: 'm-1', kind: 'messages', title: 'Inbox' });
    const detach=vi.fn();
    const mounted=renderStrip(controller);
    expect(screen.queryByRole('button',{name:'Open Inbox in a new window'})).toBeNull();mounted.unmount();
    render(()=><RealtimeContext.Provider value={controller}><SubTabStrip onDetach={detach}/></RealtimeContext.Provider>);
    fireEvent.contextMenu(screen.getByRole('tab',{name:'Inbox'}));
    fireEvent.click(screen.getByRole('menuitem',{name:'Open Inbox in a new window'}));
    expect(detach).toHaveBeenCalledWith('m-1');expect(controller.subTabs.tabs()).toHaveLength(1);
  });

  it('Arrow keys cycle the active tab', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    const a = controller.subTabs.open({ kind: 'messages', title: 'A' });
    const b = controller.subTabs.open({ kind: 'messages', title: 'B' });
    controller.subTabs.activate(a);
    renderStrip(controller);
    fireEvent.keyDown(screen.getByRole('tablist'), { key: 'ArrowRight' });
    expect(controller.subTabs.activeId()).toBe(b);
    fireEvent.keyDown(screen.getByRole('tablist'), { key: 'ArrowLeft' });
    expect(controller.subTabs.activeId()).toBe(a);
  });
});

it('shows close only for the selected tab and closes an inactive tab through its keyboard menu',()=>{
 const controller=createRealtimeController();controller.subTabs.open({id:'a',kind:'module',title:'A'});controller.subTabs.open({id:'b',kind:'module',title:'B'});renderStrip(controller);
 expect(screen.queryByRole('button',{name:'Close A'})).toBeNull();expect(screen.getByRole('button',{name:'Close B'})).toBeInTheDocument();
 fireEvent.keyDown(screen.getByRole('tab',{name:'A'}),{key:'ContextMenu'});expect(controller.subTabs.activeId()).toBe('b');
 fireEvent.click(screen.getByRole('menuitem',{name:'Close A'}));expect(controller.subTabs.tabs().map(tab=>tab.id)).toEqual(['b']);
});
it('reattaches only a detached pane and invalidates an open menu on authority change',()=>{
 activateAccountPreferences('a','msa1_verified_one');const controller=createRealtimeController();controller.subTabs.open({id:'a',kind:'module',title:'A'});const reattach=vi.fn(),detach=vi.fn();
 render(()=><RealtimeContext.Provider value={controller}><SubTabStrip onDetach={detach} isDetached={()=>true} onReattach={reattach}/></RealtimeContext.Provider>);
 fireEvent.contextMenu(screen.getByRole('tab',{name:'A'}));expect(screen.queryByRole('menuitem',{name:'Open A in a new window'})).toBeNull();
 fireEvent.click(screen.getByRole('menuitem',{name:'Reattach to main window'}));expect(reattach).toHaveBeenCalledWith('a');
 fireEvent.contextMenu(screen.getByRole('tab',{name:'A'}));activateAccountPreferences('a','msa1_verified_two');expect(screen.queryByRole('menu')).toBeNull();suspendAccountPreferences();
});

describe('SubTabStrip bulk close (t50)',()=>{
 /** Mail (home, cannot close), S (starred), A, B (active), C, D. */
 function six(onCloseMany?:(ids:string[])=>void,dir?:'rtl'){
  const controller=createRealtimeController();
  controller.subTabs.open({id:'mail',kind:'messages',title:'Mail',pinned:true});
  controller.subTabs.open({id:'s',kind:'module',title:'S',pinned:true});
  for(const id of ['a','b','c','d'])controller.subTabs.open({id,kind:'module',title:id.toUpperCase()});
  controller.subTabs.activate('b');
  render(()=><div {...(dir?{dir}:{})}><RealtimeContext.Provider value={controller}><SubTabStrip canClose={id=>id!=='mail'} {...(onCloseMany?{onCloseMany}:{})}/></RealtimeContext.Provider></div>);
  return controller;
 }
 const ids=(controller:RealtimeController)=>controller.subTabs.tabs().map(tab=>tab.id);
 const item=(name:string)=>screen.getByRole('menuitem',{name});
 it('groups the menu: closes, then star, then window, with separators between',()=>{
  six();fireEvent.contextMenu(screen.getByRole('tab',{name:'B'}));
  const menu=screen.getByRole('menu');
  expect(Array.from(menu.children).map(node=>node.getAttribute('role')==='separator'?'|':node.textContent)).toEqual(['Close B','Close other tabs','Close tabs to the left','Close tabs to the right','Close all tabs','|','Star']);
 });
 it('closes to the left and right of the middle tab, skipping the starred and Mail tabs',()=>{
  const controller=six();
  fireEvent.contextMenu(screen.getByRole('tab',{name:'B'}));fireEvent.click(item('Close tabs to the left'));
  expect(ids(controller)).toEqual(['mail','s','b','c','d']);
  fireEvent.contextMenu(screen.getByRole('tab',{name:'B'}));fireEvent.click(item('Close tabs to the right'));
  expect(ids(controller)).toEqual(['mail','s','b']);expect(controller.subTabs.activeId()).toBe('b');
 });
 it('disables a side with nothing it may close, at both ends',()=>{
  six();
  fireEvent.contextMenu(screen.getByRole('tab',{name:'A'}));
  expect(item('Close tabs to the left')).toHaveAttribute('aria-disabled','true');expect(item('Close tabs to the right')).toHaveAttribute('aria-disabled','false');
  fireEvent.click(item('Close tabs to the left'));expect(screen.getAllByRole('tab')).toHaveLength(6);
  fireEvent.contextMenu(screen.getByRole('tab',{name:'D'}));
  expect(item('Close tabs to the right')).toHaveAttribute('aria-disabled','true');expect(item('Close tabs to the left')).toHaveAttribute('aria-disabled','false');
  fireEvent.contextMenu(screen.getByRole('tab',{name:'Mail'}));
  expect(screen.queryByRole('menuitem',{name:'Close Mail'})).toBeNull();expect(item('Close tabs to the left')).toHaveAttribute('aria-disabled','true');
 });
 it('arrow keys skip the separator and the disabled items',()=>{
  six();fireEvent.keyDown(screen.getByRole('tab',{name:'D'}),{key:'ContextMenu'});
  expect(document.activeElement).toBe(item('Close D'));
  const menu=screen.getByRole('menu');
  for(const name of ['Close other tabs','Close tabs to the left','Close all tabs','Star','Close D']){fireEvent.keyDown(menu,{key:'ArrowDown'});expect(document.activeElement).toBe(item(name));}
  fireEvent.keyDown(menu,{key:'ArrowUp'});expect(document.activeElement).toBe(item('Star'));
 });
 it('in a right-to-left strip "left" means the tabs after it in reading order',()=>{
  const controller=six(undefined,'rtl');
  fireEvent.contextMenu(screen.getByRole('tab',{name:'B'}));fireEvent.click(item('Close tabs to the left'));
  expect(ids(controller)).toEqual(['mail','s','a','b']);
  fireEvent.contextMenu(screen.getByRole('tab',{name:'A'}));
  expect(item('Close tabs to the left')).toHaveAttribute('aria-disabled','false');expect(item('Close tabs to the right')).toHaveAttribute('aria-disabled','true');
 });
 it('hands the bulk close to one onCloseMany call in visual order',()=>{
  const many=vi.fn();six(many);
  fireEvent.contextMenu(screen.getByRole('tab',{name:'B'}));fireEvent.click(item('Close other tabs'));
  expect(many).toHaveBeenCalledOnce();expect(many).toHaveBeenCalledWith(['a','c','d']);
  fireEvent.contextMenu(screen.getByRole('tab',{name:'B'}));fireEvent.click(item('Close all tabs'));
  expect(many).toHaveBeenLastCalledWith(['a','b','c','d']);
 });
});
