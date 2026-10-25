import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { ImpactPreview, IMPACT_NAMES_SHOWN } from './ImpactPreview.tsx';
import type { ImpactItem } from './adminApi.ts';
import { stripIsolates } from '../../../i18n';
import { expectNoAxeViolations, problem } from './uxTestkit.ts';

const LABELS = {
  usersRefused: (n: number) => (n === 1 ? 'User refused' : 'Users refused'),
  sessionsEnded: () => 'Sessions ended',
};

function Harness(props: {
  load: () => Promise<Record<string, ImpactItem> | null>;
  onConfirm?: () => void | Promise<void>;
  typed?: string;
}) {
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Switch mode
      </button>
      <ImpactPreview
        open={open()}
        title="Switch example.org to listed mode?"
        loadImpact={props.load}
        labels={LABELS}
        confirmLabel="Switch to listed"
        {...(props.typed ? { typedConfirmation: props.typed } : {})}
        onConfirm={async () => {
          await props.onConfirm?.();
          setOpen(false);
        }}
        onCancel={() => setOpen(false)}
      />
    </>
  );
}

const names = (n: number) => Array.from({ length: n }, (_, i) => `user${i + 1}@example.org`);

describe('ImpactPreview', () => {
  it('runs the dry run, lists counts and the first 20 names, and says how many more', async () => {
    const load = vi.fn(async () => ({ usersRefused: { count: 25, names: names(25) }, sessionsEnded: { count: 3 } }));
    render(() => <Harness load={load} />);
    fireEvent.click(screen.getByRole('button', { name: 'Switch mode' }));
    const dlg = await screen.findByRole('alertdialog', { name: 'Switch example.org to listed mode?' });
    const users = await within(dlg).findByText(/Users refused: 25/);
    const item = users.closest('li')!;
    expect(item).toHaveAttribute('data-impact', 'usersRefused');
    const listed = stripIsolates(item.textContent ?? '');
    expect(listed).toContain('user20@example.org');
    expect(listed).not.toContain('user21@example.org');
    expect(listed).toContain('and 5 more');
    expect(IMPACT_NAMES_SHOWN).toBe(20);
    expect(within(dlg).getByText('Sessions ended: 3')).toBeInTheDocument();
    expect(load).toHaveBeenCalledTimes(1);
  });

  it('Confirm waits for the dry run; a destructive change needs the name typed', async () => {
    let finish!: (v: Record<string, ImpactItem>) => void;
    const confirm = vi.fn();
    render(() => <Harness load={() => new Promise((r) => (finish = r))} onConfirm={confirm} typed="example.org" />);
    fireEvent.click(screen.getByRole('button', { name: 'Switch mode' }));
    const button = await screen.findByRole('button', { name: 'Switch to listed' });
    expect(screen.getByRole('status')).toHaveTextContent('Checking what this change affects');
    expect(button).toBeDisabled();
    finish({ usersRefused: { count: 1, names: ['a@example.org'] } });
    await screen.findByText('User refused: 1');
    expect(button).toBeDisabled();
    fireEvent.input(screen.getByRole('textbox'), { target: { value: 'example.org' } });
    expect(button).toBeEnabled();
    fireEvent.click(button);
    await waitFor(() => expect(confirm).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
  });

  it('says plainly when the server returned no preview, and still lets the admin confirm', async () => {
    render(() => <Harness load={async () => null} />);
    fireEvent.click(screen.getByRole('button', { name: 'Switch mode' }));
    expect(await screen.findByText(/Impact preview not available/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Switch to listed' })).toBeEnabled();
  });

  it('says nothing else changes when every count is zero', async () => {
    render(() => <Harness load={async () => ({ usersRefused: { count: 0 } })} />);
    fireEvent.click(screen.getByRole('button', { name: 'Switch mode' }));
    expect(await screen.findByText('Nothing else changes.')).toBeInTheDocument();
  });

  it('a failed dry run shows the reason and Try again; Confirm stays off until it answers', async () => {
    let calls = 0;
    const load = vi.fn(async () => {
      calls += 1;
      if (calls === 1) throw problem(403, { code: 'capability_missing' });
      return { sessionsEnded: { count: 2 } };
    });
    render(() => <Harness load={load} />);
    fireEvent.click(screen.getByRole('button', { name: 'Switch mode' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Your role does not allow this');
    expect(screen.getByRole('button', { name: 'Switch to listed' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(await screen.findByText('Sessions ended: 2')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Switch to listed' })).toBeEnabled();
  });

  it('Escape cancels, nothing is written, focus returns to the opener', async () => {
    const confirm = vi.fn();
    render(() => <Harness load={async () => ({})} onConfirm={confirm} />);
    const opener = screen.getByRole('button', { name: 'Switch mode' });
    opener.focus();
    fireEvent.click(opener);
    const dlg = await screen.findByRole('alertdialog');
    fireEvent.keyDown(dlg, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
    expect(confirm).not.toHaveBeenCalled();
    expect(document.activeElement).toBe(opener);
  });

  it('has no axe violations with the impact listed', async () => {
    render(() => <Harness load={async () => ({ usersRefused: { count: 2, names: names(2) } })} typed="example.org" />);
    fireEvent.click(screen.getByRole('button', { name: 'Switch mode' }));
    const dlg = await screen.findByRole('alertdialog');
    await within(dlg).findByText(/Users refused: 2/);
    await expectNoAxeViolations(dlg);
  });
});
