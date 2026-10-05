import { describe, it, expect, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { MessageActions } from './MessageActions.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';
import * as css from './messageRow.css.ts';

// jsdom computes no layout and evaluates no media query, so what is tested here
// is the structure and state the stylesheet keys on. The rendered geometry at
// desktop and phone widths is covered in a browser (e2e/narrow.spec.ts,
// e2e/modern-ux.spec.ts).

describe('MessageActions', () => {
  beforeEach(() => localStorage.clear());

  function mount() {
    const out = renderWithApp(() => <MessageActions email={mkEmail('a')} />);
    const toggle = screen.getByRole('button', { name: 'More actions' });
    const cluster = screen.getByRole('group', { name: 'More actions' });
    return { ...out, toggle, cluster };
  }

  it('keeps the toggle outside the cluster it opens', () => {
    const { toggle, cluster } = mount();
    expect(cluster).toHaveClass('msg-actions');
    expect(cluster.contains(toggle)).toBe(false);
    expect(
      within(cluster)
        .getAllByRole('button')
        .map((b) => b.getAttribute('aria-label')),
    ).toEqual(['Pin', 'Snooze', 'Label', 'Flag for follow-up', 'Archive', 'Delete']);
  });

  it('the toggle opens and closes the cluster', () => {
    const { toggle, cluster } = mount();
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(cluster).not.toHaveClass(css.actionsOpen);
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    expect(cluster).toHaveClass(css.actionsOpen);
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(cluster).not.toHaveClass(css.actionsOpen);
  });

  it('an open menu keeps the cluster shown without the toggle, and Escape closes it', () => {
    const { cluster } = mount();
    const snooze = within(cluster).getByRole('button', { name: 'Snooze' });
    expect(snooze).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(snooze);
    expect(snooze).toHaveAttribute('aria-expanded', 'true');
    expect(cluster).toHaveClass(css.actionsOpen);
    expect(within(cluster).getByRole('menu', { name: 'Snooze until' })).toBeInTheDocument();

    fireEvent.keyDown(snooze, { key: 'Escape' });
    expect(within(cluster).queryByRole('menu')).toBeNull();
    expect(snooze).toHaveAttribute('aria-expanded', 'false');
    expect(cluster).not.toHaveClass(css.actionsOpen);
  });

  it('Escape closes the menu first and the cluster second', () => {
    const { toggle, cluster } = mount();
    fireEvent.click(toggle);
    fireEvent.click(within(cluster).getByRole('button', { name: 'Label' }));
    expect(within(cluster).getByRole('menu', { name: 'Labels' })).toBeInTheDocument();

    fireEvent.keyDown(cluster, { key: 'Escape' });
    expect(within(cluster).queryByRole('menu')).toBeNull();
    expect(cluster).toHaveClass(css.actionsOpen);
    fireEvent.keyDown(cluster, { key: 'Escape' });
    expect(cluster).not.toHaveClass(css.actionsOpen);
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes when focus moves somewhere else, and stays open while it moves inside', () => {
    const { toggle, cluster } = mount();
    fireEvent.click(toggle);
    const pin = within(cluster).getByRole('button', { name: 'Pin' });
    fireEvent.focusOut(toggle, { relatedTarget: pin });
    expect(cluster).toHaveClass(css.actionsOpen);
    fireEvent.focusOut(pin, { relatedTarget: document.body });
    expect(cluster).not.toHaveClass(css.actionsOpen);
  });

  it('opening from the toggle puts focus on the first action; Escape hands it back', () => {
    const { toggle, cluster } = mount();
    toggle.focus();
    fireEvent.click(toggle);
    expect(cluster).toHaveClass(css.actionsOpen);
    expect(document.activeElement).toBe(within(cluster).getByRole('button', { name: 'Pin' }));

    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(cluster).not.toHaveClass(css.actionsOpen);
    expect(document.activeElement).toBe(toggle);
  });

  it('a menu opened without the toggle does not move focus to the first action', () => {
    const { cluster } = mount();
    const snooze = within(cluster).getByRole('button', { name: 'Snooze' });
    snooze.focus();
    fireEvent.click(snooze);
    expect(document.activeElement).toBe(snooze);
  });

  it('a message without a follow-up carries no mark', () => {
    const { cluster } = mount();
    expect(screen.queryByTestId('msg-followup-mark')).toBeNull();
    const flag = within(cluster).getByRole('button', { name: 'Flag for follow-up' });
    expect(flag).toHaveAttribute('aria-pressed', 'false');
  });

  it('shows the mark, outside the cluster, for a message that has a follow-up', () => {
    renderWithApp(() => <MessageActions email={{ ...mkEmail('b'), followUpAt: '2030-01-01T09:00:00.000Z' }} />);
    const cluster = screen.getByRole('group', { name: 'More actions' });
    const mark = screen.getByTestId('msg-followup-mark');
    expect(mark).toHaveAttribute('role', 'img');
    expect(mark).toHaveAccessibleName('Flag for follow-up');
    expect(cluster.contains(mark)).toBe(false);
    expect(within(cluster).getByRole('button', { name: 'Clear follow-up' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('flagging from the toggled cluster closes it and hands focus back to the toggle', async () => {
    const { app, toggle, cluster } = mount();
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    fireEvent.click(toggle);
    fireEvent.click(within(cluster).getByRole('button', { name: 'Flag for follow-up' }));
    expect(cluster).not.toHaveClass(css.actionsOpen);
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(document.activeElement).toBe(toggle);
  });

  it('picking a snooze preset snoozes the message and closes everything', async () => {
    const { app, toggle, cluster } = mount();
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    fireEvent.click(toggle);
    fireEvent.click(within(cluster).getByRole('button', { name: 'Snooze' }));
    fireEvent.click(within(cluster).getByRole('menuitem', { name: 'Tomorrow' }));
    expect(within(cluster).queryByRole('menu')).toBeNull();
    expect(cluster).not.toHaveClass(css.actionsOpen);
    await waitFor(() => expect(app.pendingUndo()?.label).toBe('Snoozed'));
  });
});

describe('app.css — the per-row action cluster', () => {
  it('no longer hides the cluster in the narrow block', () => {
    const appCss = readFileSync(resolve(process.cwd(), 'src/styles/app.css'), 'utf8');
    expect(appCss).not.toMatch(/\.msg-actions/);
  });
});
