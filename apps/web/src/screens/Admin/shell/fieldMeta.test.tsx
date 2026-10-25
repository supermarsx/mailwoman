// RevertToDefault, FieldHistory and StaleNotice (UX7, UX8, UX19).
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { RevertToDefault } from './RevertToDefault.tsx';
import { FieldHistory, historyPath } from './FieldHistory.tsx';
import { StaleNotice } from './StaleNotice.tsx';
import { renderInShell } from './testkit.tsx';
import { stripIsolates } from '../../../i18n';
import { expectNoAxeViolations, problem } from './uxTestkit.ts';

beforeEach(() => history.replaceState(null, '', '/admin'));

describe('RevertToDefault', () => {
  it('names the default; the button deletes the stored value and announces the result', async () => {
    const onRevert = vi.fn(async () => undefined);
    render(() => <RevertToDefault label="Port" defaultDisplay="587" isDefault={false} onRevert={onRevert} />);
    expect(stripIsolates(screen.getByText(/Default:/).textContent ?? '')).toBe('Default: 587');
    const button = screen.getByRole('button', { name: 'Revert to default (Port)' });
    expect(button).toHaveAccessibleDescription(/Default: .*587/);
    fireEvent.click(button);
    await waitFor(() => expect(onRevert).toHaveBeenCalledTimes(1));
    expect(stripIsolates((await screen.findByRole('status')).textContent ?? '')).toBe('Reverted to the default (587).');
  });

  it('shows no button while the default is in use', () => {
    render(() => <RevertToDefault label="Port" defaultDisplay="587" isDefault onRevert={() => undefined} />);
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('is disabled with the reason when the value is pinned', () => {
    render(() => (
      <RevertToDefault label="Port" defaultDisplay="587" isDefault={false} onRevert={() => undefined} disabledReason="Set by MW_PORT" />
    ));
    expect(screen.getByRole('button', { name: /Revert to default/ })).toBeDisabled();
    expect(screen.getByRole('button', { name: /Revert to default/ })).toHaveAttribute('title', 'Set by MW_PORT');
  });

  it('a refused revert says what failed', async () => {
    render(() => (
      <RevertToDefault label="Port" defaultDisplay="587" isDefault={false} onRevert={async () => Promise.reject(problem(403, { code: 'capability_missing' }))} />
    ));
    fireEvent.click(screen.getByRole('button', { name: /Revert to default/ }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Your role does not allow this');
  });
});

describe('FieldHistory', () => {
  it('says who changed it and when, and links the filtered audit log', () => {
    render(() => <FieldHistory target="setting:signin.newUsers" updatedBy="alice" updatedAt="2026-10-25T09:30:00Z" />);
    const text = stripIsolates(screen.getByTestId('field-history').textContent ?? '');
    expect(text).toMatch(/^Last changed by alice on .*2026/);
    const link = screen.getByRole('link', { name: 'History' });
    expect(link).toHaveAttribute('href', '/admin#/audit?target=setting%3Asignin.newUsers');
    expect(historyPath('domain:x.org')).toBe('/admin#/audit?target=domain%3Ax.org');
  });

  it('says so when the value was never changed from the console', () => {
    render(() => <FieldHistory target="setting:x" />);
    expect(screen.getByText('Not changed from the console')).toBeInTheDocument();
  });

  it('inside the console a plain click navigates in place through the router', () => {
    const { navigate } = renderInShell(() => <FieldHistory target="setting:x" updatedBy="bob" />);
    fireEvent.click(screen.getByRole('link', { name: 'History' }));
    expect(navigate).toHaveBeenCalledWith('/admin#/audit?target=setting%3Ax');
  });
});

describe('StaleNotice', () => {
  it('says who changed it and when, with Reload and Keep mine', async () => {
    const reload = vi.fn();
    const keep = vi.fn();
    const { container } = render(() => (
      <StaleNotice conflict={{ updatedBy: 'bob', updatedAt: '2026-10-25T10:00:00Z' }} onReload={reload} onKeepMine={keep} />
    ));
    const alert = screen.getByRole('alert');
    expect(stripIsolates(alert.textContent ?? '')).toMatch(/^bob changed this on .* while you were editing\./);
    fireEvent.click(screen.getByRole('button', { name: 'Reload' }));
    await waitFor(() => expect(reload).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Keep mine' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Keep mine' }));
    await waitFor(() => expect(keep).toHaveBeenCalledTimes(1));
    await expectNoAxeViolations(container);
  });

  it('renders nothing without a conflict, and a neutral sentence when the server named nobody', () => {
    const { unmount } = render(() => <StaleNotice conflict={null} onReload={() => undefined} onKeepMine={() => undefined} />);
    expect(screen.queryByRole('alert')).toBeNull();
    unmount();
    render(() => <StaleNotice conflict={{ updatedBy: null, updatedAt: null }} onReload={() => undefined} onKeepMine={() => undefined} />);
    expect(screen.getByRole('alert')).toHaveTextContent('Someone else changed this while you were editing.');
  });
});
