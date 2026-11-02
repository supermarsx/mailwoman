import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { createContextMenu } from './ContextMenu.tsx';
const frames: HTMLIFrameElement[] = [];
afterEach(() => { cleanup(); frames.splice(0).forEach(frame => frame.remove()); });
function childDocument(): Document {
  const frame = document.createElement('iframe'); document.body.append(frame); frames.push(frame);
  return frame.contentDocument!;
}
describe('context menu owning document', () => {
  it('mounts/focuses in its owner and handles Escape and outside clicks there', () => {
    const doc = childDocument(), trigger = doc.createElement('button'); trigger.textContent = 'Address'; doc.body.append(trigger);
    let open!: () => void;
    render(() => {
      const menu = createContextMenu({ ownerDocument: () => doc, label: () => 'Contact actions', scope: () => 'a', actions: () => [{ id: 'card', label: 'Contact details', icon: 'person', run: () => {} }] });
      open = () => menu.openAt({ x: 40, y: 50, trigger }); return menu.menu();
    });
    trigger.focus(); open();
    expect(within(document.body).queryByRole('menu')).toBeNull();
    let menu = within(doc.body).getByRole('menu');
    expect(doc.activeElement).toBe(within(menu).getByRole('menuitem'));
    fireEvent.keyDown(menu, { key: 'Escape' });
    expect(within(doc.body).queryByRole('menu')).toBeNull(); expect(doc.activeElement).toBe(trigger);
    open(); menu = within(doc.body).getByRole('menu');
    fireEvent.pointerDown(document.body);
    expect(within(doc.body).getByRole('menu')).toBe(menu);
    fireEvent.pointerDown(doc.body);
    expect(within(doc.body).queryByRole('menu')).toBeNull();
  });
  it('closes on owner replacement and removes the original document subscriptions', () => {
    const doc = childDocument(), childTrigger = doc.createElement('button'); doc.body.append(childTrigger);
    const rootTrigger = document.createElement('button'); document.body.append(rootTrigger);
    let changeOwner!: (doc: Document) => void, open!: (trigger: HTMLElement) => void;
    const view = render(() => {
      const [owner, setOwner] = createSignal(doc); changeOwner = setOwner;
      const menu = createContextMenu({ ownerDocument: owner, label: () => 'Contact actions', scope: () => 'a', actions: () => [{ id: 'card', label: 'Contact details', icon: 'person', run: () => {} }] });
      open = trigger => menu.openAt({ x: 40, y: 50, trigger }); return menu.menu();
    });
    open(childTrigger); expect(doc.body.contains(within(doc.body).getByRole('menu'))).toBe(true);
    changeOwner(document); expect(within(doc.body).queryByRole('menu')).toBeNull();
    open(rootTrigger); const current = within(document.body).getByRole('menu');
    fireEvent.pointerDown(doc.body); expect(within(document.body).getByRole('menu')).toBe(current);
    view.unmount(); expect(within(document.body).queryByRole('menu')).toBeNull(); rootTrigger.remove();
  });
});
