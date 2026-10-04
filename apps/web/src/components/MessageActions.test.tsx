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
