import { describe, it, expect, vi } from 'vitest';
import { createSubTabs } from './subTabs.ts';

describe('createSubTabs', () => {
  it('reorders stable tab identities without activating or losing pin and payload state',()=>{
    const model=createSubTabs();model.open({id:'a',kind:'module',title:'Contacts',pinned:true,data:{surface:'contacts'}});model.open({id:'b',kind:'messages',title:'Message'});
    const original=model.tabs()[0];model.move('a',1);
    expect(model.tabs().map(tab=>tab.id)).toEqual(['b','a']);expect(model.activeId()).toBe('b');expect(model.tabs()[1]).toBe(original);
    model.move('missing',0);model.move('a',Number.NaN);expect(model.tabs().map(tab=>tab.id)).toEqual(['b','a']);
    model.move('a',-50);expect(model.tabs()[0]?.id).toBe('a');
  });
  it('renames only the indicated tab and bounds labels',()=>{
    const model=createSubTabs();model.open({id:'a',kind:'composer',title:'New message'});model.rename('a','Subject');expect(model.tabs()[0]?.title).toBe('Subject');
    model.rename('a','x'.repeat(1000));expect(model.tabs()[0]?.title).toHaveLength(512);
  });
  it('opens a tab, makes it active, and returns its id', () => {
    const s = createSubTabs();
    const id = s.open({ kind: 'messages', title: 'Inbox' });
    expect(s.tabs()).toHaveLength(1);
    expect(s.activeId()).toBe(id);
    expect(s.tabs()[0]!.title).toBe('Inbox');
  });

  it('reuses an existing tab by id instead of duplicating', () => {
    const s = createSubTabs();
    const a = s.open({ id: 'draft-1', kind: 'composer', title: 'Draft' });
    s.open({ kind: 'messages', title: 'Inbox' });
    const b = s.open({ id: 'draft-1', kind: 'composer', title: 'Draft' });
    expect(a).toBe(b);
    expect(s.tabs()).toHaveLength(2);
    expect(s.activeId()).toBe('draft-1');
  });

  it('closing the active tab returns to the last focused tab rather than the left neighbour', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'messages', title: 'A' });
    const b = s.open({ kind: 'messages', title: 'B' });
    const c = s.open({ kind: 'messages', title: 'C' });
    s.activate(b);
    s.close(b);
    expect(s.tabs().map((t) => t.id)).toEqual([a, c]);
    expect(s.activeId()).toBe(c);
  });

  it('walks all surviving focus history across tab kinds and ignores background closures and reordering',()=>{
    const s=createSubTabs();
    s.open({id:'mail',kind:'messages',title:'Mail',pinned:true});
    s.open({id:'settings',kind:'settings',title:'Settings'});
    s.open({id:'draft',kind:'composer',title:'Draft'});
    s.open({id:'pdf',kind:'attachment',title:'PDF'});
    s.activate('settings');s.activate('mail');s.activate('draft');
    s.move('mail',3);s.close('pdf');
    expect(s.activeId()).toBe('draft');
    s.close('draft');expect(s.activeId()).toBe('mail');
    s.close('mail');expect(s.activeId()).toBe('settings');
    s.close('settings');expect(s.activeId()).toBeNull();
  });

  it('records keyboard cycling and reusing a tab in focus history',()=>{
    const s=createSubTabs();
    for(const id of ['a','b','c'])s.open({id,kind:'messages',title:id});
    s.activate('a');s.cycle(1);s.close('b');expect(s.activeId()).toBe('a');
    s.open({id:'c',kind:'messages',title:'Existing'});s.close('c');expect(s.activeId()).toBe('a');
  });

  it('closing the last remaining tab clears the active id', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'settings', title: 'Settings' });
    s.close(a);
    expect(s.tabs()).toHaveLength(0);
    expect(s.activeId()).toBeNull();
  });

  it('togglePin flips the pinned flag', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'messages', title: 'A' });
    expect(s.tabs()[0]!.pinned).toBe(false);
    s.togglePin(a);
    expect(s.tabs()[0]!.pinned).toBe(true);
  });

  it('cycle moves focus and wraps in both directions', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'messages', title: 'A' });
    const b = s.open({ kind: 'messages', title: 'B' });
    s.activate(a);
    expect(s.cycle(1)).toBe(b);
    expect(s.cycle(1)).toBe(a); // wrap forward
    expect(s.cycle(-1)).toBe(b); // wrap backward
  });

  it('cycle returns null when there are no tabs', () => {
    const s = createSubTabs();
    expect(s.cycle(1)).toBeNull();
  });

  it('tearOff opens a window with the tab url and closes it locally', () => {
    const openWindow = vi.fn(() => ({}));
    const s = createSubTabs({ openWindow });
    const a = s.open({ id: 'm-1', kind: 'messages', title: 'Inbox' });
    s.tearOff(a);
    expect(openWindow).toHaveBeenCalledWith('?tab=m-1', '_blank');
    expect(s.tabs()).toHaveLength(0);
  });
});
