// PinnedBadge, LockSwitch, Breadcrumbs, RestorePrevious.
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, within, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { PinnedBadge } from './PinnedBadge.tsx';
import { LockSwitch } from './LockSwitch.tsx';
import { Breadcrumbs } from './Breadcrumbs.tsx';
import { RestorePrevious } from './RestorePrevious.tsx';
import { isolate } from '../../../i18n';
import { AdminApiError } from '../../../state/slices/admin.ts';

describe('PinnedBadge', () => {
  it('names the environment variable and explains why the control is disabled', () => {
    render(() => (
      <>
        <select aria-label="Log level" disabled aria-describedby="pin" />
        <PinnedBadge id="pin" envVar="MW_LOG_LEVEL" />
      </>
    ));
    expect(screen.getByTestId('pinned-badge')).toHaveTextContent('Set by MW_LOG_LEVEL');
    expect(screen.getByRole('combobox', { name: 'Log level' })).toHaveAccessibleDescription(
      "Set by MW_LOG_LEVEL This value comes from the server's environment and cannot be changed here.",
    );
  });
});

describe('LockSwitch', () => {
  it('is a switch with a stable name; the state is aria-checked', () => {
    const [locked, setLocked] = createSignal(false);
    render(() => <LockSwitch locked={locked()} onChange={setLocked} settingLabel="Theme" />);
    const sw = screen.getByRole('switch', { name: 'Lock for users Theme' });
    expect(sw).toHaveAttribute('aria-checked', 'false');
    fireEvent.click(sw);
    expect(sw).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('switch', { name: 'Lock for users Theme' })).toBe(sw);
  });

  it('disabled: does not change, and says why', () => {
    const change = vi.fn();
    render(() => (
      <>
        <LockSwitch locked={false} onChange={change} settingLabel="Theme" disabled describedBy="why" />
        <p id="why">Your role does not include settings.write.</p>
      </>
    ));
    const sw = screen.getByRole('switch');
    expect(sw).toBeDisabled();
    expect(sw).toHaveAccessibleDescription('Your role does not include settings.write.');
    fireEvent.click(sw);
    expect(change).not.toHaveBeenCalled();
  });
});

describe('Breadcrumbs', () => {
  it('an ordered trail; earlier items navigate in place, the last is the current page', () => {
    const go = vi.fn();
    render(() => (
      <Breadcrumbs
        items={[
          { label: 'Dashboard', href: '/admin', onNavigate: go },
          { label: 'Domains', href: '/admin#/domains' },
          { label: 'example.org' },
        ]}
      />
    ));
    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    const items = within(nav).getAllByRole('listitem');
    expect(items.map((i) => i.textContent)).toEqual(['Dashboard', 'Domains', 'example.org']);
    expect(within(items[2]!).queryByRole('link')).toBeNull();
    expect(within(items[2]!).getByText('example.org')).toHaveAttribute('aria-current', 'page');
    fireEvent.click(within(nav).getByRole('link', { name: 'Dashboard' }));
    expect(go).toHaveBeenCalled();
  });

  it('a modified click is left to the browser (new tab)', () => {
    const go = vi.fn();
    render(() => <Breadcrumbs items={[{ label: 'Dashboard', href: '/admin', onNavigate: go }, { label: 'X' }]} />);
    // Runs after the component's handler (Solid delegates to document first); it only
    // stops jsdom's unimplemented navigation, and records whether the component left
    // the default alone.
    let leftToBrowser = false;
    document.addEventListener(
      'click',
      (e) => {
        leftToBrowser = !e.defaultPrevented;
        e.preventDefault();
      },
      { once: true },
    );
    fireEvent.click(screen.getByRole('link', { name: 'Dashboard' }), { ctrlKey: true });
    expect(go).not.toHaveBeenCalled();
    expect(leftToBrowser).toBe(true);
  });

  it('renders nothing without items', () => {
    render(() => <Breadcrumbs items={[]} />);
    expect(screen.queryByRole('navigation')).toBeNull();
  });
});

describe('RestorePrevious', () => {
  const prev = { display: 'Dark', actor: 'root', changedAt: '23 Oct 2026, 21:00' };

  it('shows who changed it, when, and the previous value; restore writes and confirms', async () => {
    const restore = vi.fn(async () => undefined);
    render(() => <RestorePrevious previous={prev} onRestore={restore} />);
    expect(screen.getByTestId('restore-previous')).toHaveTextContent(`Changed by ${isolate('root')} on 23 Oct 2026, 21:00. Previous value: ${isolate('Dark')}`);
    fireEvent.click(screen.getByRole('button', { name: 'Restore previous value' }));
    await waitFor(() => expect(restore).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole('status')).toHaveTextContent('The previous value is restored.');
  });

  it('a failed restore says what failed', async () => {
    render(() => <RestorePrevious previous={prev} onRestore={async () => Promise.reject(new AdminApiError(409, 'x'))} />);
    fireEvent.click(screen.getByRole('button', { name: 'Restore previous value' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Someone changed it in the meantime');
  });

  it('a redacted secret is shown but not offered for restore; no record shows nothing', () => {
    const { unmount } = render(() => <RestorePrevious previous={{ ...prev, display: '(secret)', restorable: false }} onRestore={vi.fn()} />);
    expect(screen.queryByRole('button', { name: 'Restore previous value' })).toBeNull();
    unmount();
    render(() => <RestorePrevious previous={null} onRestore={vi.fn()} />);
    expect(screen.queryByTestId('restore-previous')).toBeNull();
  });
});
