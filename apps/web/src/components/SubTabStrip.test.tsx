import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import { toasts, resetToasts } from './toast/index.ts';
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { SubTabStrip } from './SubTabStrip.tsx';
import { RealtimeContext } from '../realtime/context.ts';
import { createRealtimeController, type RealtimeController } from '../realtime/controller.ts';

function renderStrip(controller: RealtimeController) {
  return render(() => (
    <RealtimeContext.Provider value={controller}>
      <SubTabStrip />
    </RealtimeContext.Provider>
  ));
}

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
