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

  it('the pin button toggles pinned state', () => {
    const controller = createRealtimeController({ subTabs: { openWindow: vi.fn() } });
    controller.subTabs.open({ kind: 'messages', title: 'Inbox' });
    renderStrip(controller);
    fireEvent.click(screen.getByRole('button', { name: 'Pin Inbox' }));
    expect(controller.subTabs.tabs()[0]!.pinned).toBe(true);
    // The label flips to the unpin affordance.
    expect(screen.getByRole('button', { name: 'Unpin Inbox' })).toBeInTheDocument();
  });

  it('the tear-off button opens a window and closes the tab locally', () => {
    const openWindow = vi.fn(() => ({}));
    const controller = createRealtimeController({ subTabs: { openWindow } });
    controller.subTabs.open({ id: 'm-1', kind: 'messages', title: 'Inbox' });
    renderStrip(controller);
    fireEvent.click(screen.getByRole('button', { name: 'Open Inbox in a new window' }));
    expect(openWindow).toHaveBeenCalledWith('?tab=m-1', '_blank');
    expect(screen.queryByRole('tab')).toBeNull();
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
