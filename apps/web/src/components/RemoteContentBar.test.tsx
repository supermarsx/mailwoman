import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { RemoteContentBar } from './RemoteContentBar.tsx';
import type { BlockedContentReport, RemoteImageGrant } from '../api/remote-images.ts';

function report(over: Partial<BlockedContentReport> = {}): BlockedContentReport {
  return { blockedHosts: ['cdn.example', 'tracker.evil'], blockedCount: 3, trackerCount: 0, ...over };
}

const CTX = { emailId: 'M1', sender: 'bob@spam.example' } as const;

describe('RemoteContentBar — blocked state', () => {
  it('shows the remote-image count and the blocked hosts', () => {
    render(() => <RemoteContentBar {...CTX} report={report()} />);
    expect(screen.getByText('3 remote images blocked')).toBeInTheDocument();
    expect(screen.getByText('cdn.example')).toBeInTheDocument();
    expect(screen.getByText('tracker.evil')).toBeInTheDocument();
  });

  it('frames the count as trackers when the sanitizer classified any', () => {
    render(() => <RemoteContentBar {...CTX} report={report({ trackerCount: 2 })} />);
    // "2 trackers blocked of 3 remote images"
    expect(screen.getByText(/2 trackers blocked/)).toBeInTheDocument();
    expect(screen.getByText(/of 3 remote images/)).toBeInTheDocument();
  });

  it('renders all 4 grant actions with the sender / domain named', () => {
    render(() => <RemoteContentBar {...CTX} report={report()} />);
    expect(screen.getByRole('button', { name: 'Load images' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Always load from bob@spam.example' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Always load from spam.example' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Always load all remote images' })).toBeInTheDocument();
  });

  // Each action, isolated (buttons disable while an action is pending, so a real
  // click is followed by a reload rather than a second grant).
  const SCOPES: [string, { kind: string; value: string }][] = [
    ['Load images', { kind: 'single', value: 'M1' }],
    ['Always load from bob@spam.example', { kind: 'per-sender', value: 'bob@spam.example' }],
    ['Always load from spam.example', { kind: 'per-domain', value: 'spam.example' }],
    ['Always load all remote images', { kind: 'all', value: '' }],
  ];
  for (const [label, scope] of SCOPES) {
    it(`"${label}" dispatches the ${scope.kind} scope`, () => {
      const onGrant = vi.fn().mockResolvedValue(undefined);
      const { unmount } = render(() => <RemoteContentBar {...CTX} report={report()} onGrant={onGrant} />);
      fireEvent.click(screen.getByRole('button', { name: label }));
      expect(onGrant).toHaveBeenCalledWith(scope);
      unmount();
    });
  }

  it('confirms the action in the live status region', async () => {
    render(() => <RemoteContentBar {...CTX} report={report()} />);
    fireEvent.click(screen.getByRole('button', { name: 'Load images' }));
    expect(await screen.findByText('Remote images loaded for this message.')).toBeInTheDocument();
  });

  it('hides per-sender / per-domain actions when the sender is unknown', () => {
    render(() => <RemoteContentBar emailId="M1" sender="" report={report()} />);
    expect(screen.getByRole('button', { name: 'Load images' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Always load all remote images' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Always load from/ })).toBeNull();
  });

  it('hides per-domain when the sender has no domain', () => {
    render(() => <RemoteContentBar emailId="M1" sender="local-only" report={report()} />);
    expect(screen.getByRole('button', { name: 'Always load from local-only' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Always load from ' })).toBeNull();
  });

  it('falls back to a no-op when no onGrant is supplied', async () => {
    render(() => <RemoteContentBar {...CTX} report={report()} />);
    fireEvent.click(screen.getByRole('button', { name: 'Always load all remote images' }));
    expect(await screen.findByText('Remote images will load for all mail.')).toBeInTheDocument();
  });
});

describe('RemoteContentBar — allowed state', () => {
  const activeGrant: RemoteImageGrant = {
    scopeKind: 'per-domain',
    scopeValue: 'spam.example',
    grantedAt: '2026-07-19T00:00:00Z',
  };

  it('shows the allowed note and a turn-off control (no grant buttons)', () => {
    render(() => <RemoteContentBar {...CTX} report={report({ blockedCount: 0, blockedHosts: [] })} activeGrant={activeGrant} />);
    expect(screen.getByText('Remote images are loading for this message.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Turn off' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Load images' })).toBeNull();
  });

  it('revoke dispatches the covering grant scope', () => {
    const onRevoke = vi.fn().mockResolvedValue(undefined);
    render(() => (
      <RemoteContentBar {...CTX} report={report()} activeGrant={activeGrant} onRevoke={onRevoke} />
    ));
    fireEvent.click(screen.getByRole('button', { name: 'Turn off' }));
    expect(onRevoke).toHaveBeenCalledWith({ kind: 'per-domain', value: 'spam.example' });
  });
});
