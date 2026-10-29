import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { AuthLogs, type AuthLogsApi } from './AuthLogs.tsx';
import { renderInShell } from './shell/testkit.tsx';
import { expectNoAxeViolations } from './shell/uxTestkit.ts';
const event = { id: 'event1', accountId: null, usernameAttempted: 'unknown@example.org', ts: 100, lastSeen: 102, repeatCount: 4, result: 'failure', method: 'password', ip: '192.0.2.1', userAgent: '<script>bad</script>', reason: 'auth-rejected', surface: 'webmail', backend: 'jmap' };
describe('authentication history', () => {
    it('shows attempt metadata and preserves filters while paging', async () => {
        const history = vi.fn<AuthLogsApi['history']>(async (params) => ({ kind: 'ok', etag: null, data: { events: params.has('cursor') ? [] : [event], cursor: params.has('cursor') ? null : '102:event1', retentionDays: 7 } }));
        const { container } = renderInShell(() => <AuthLogs api={{ history }}/>);
        expect(await screen.findByText('unknown@example.org')).toBeInTheDocument();
        expect(screen.getByRole('cell', { name: '4' })).toBeInTheDocument();
        fireEvent.input(screen.getByLabelText('Account, attempted username or IP'), { target: { value: 'unknown' } });
        fireEvent.change(screen.getByLabelText('Sign-in area'), { target: { value: 'webmail' } });
        fireEvent.click(screen.getByRole('button', { name: 'Apply filters' }));
        await vi.waitFor(() => expect(history).toHaveBeenCalledTimes(2));
        fireEvent.click(screen.getByRole('button', { name: 'Older attempts' }));
        await vi.waitFor(() => expect(history).toHaveBeenCalledTimes(3));
        expect(history.mock.calls[2]![0].get('q')).toBe('unknown');
        expect(history.mock.calls[2]![0].get('surface')).toBe('webmail');
        expect(history.mock.calls[2]![0].get('cursor')).toBe('102:event1');
        expect(container.querySelector('script')).toBeNull();
        await expectNoAxeViolations(container);
    });
});
describe('authentication history actor links', () => {
    it('links a resolved account to its page and leaves unresolved attempts as text', async () => {
        const linked = { ...event, id: 'event2', accountId: 'acct-1', usernameAttempted: 'ana@example.test', result: 'success', reason: '', actorLabel: 'Ana (ana@example.test)', actorLink: { section: 'users' as const, id: 'acct-1' } };
        const alias = { ...linked, id: 'event3', usernameAttempted: 'ana.alias@example.test' };
        const history = vi.fn<AuthLogsApi['history']>(async () => ({ kind: 'ok', etag: null, data: { events: [linked, alias, event], cursor: null, retentionDays: 7 } }));
        const { container } = renderInShell(() => <AuthLogs api={{ history }}/>);
        const links = await screen.findAllByRole('link', { name: 'Ana (ana@example.test)' });
        expect(links[0]).toHaveAttribute('href', expect.stringContaining('users/acct-1'));
        expect(screen.getByText('Attempted as ana.alias@example.test')).toBeInTheDocument();
        expect(screen.queryByText('Attempted as ana@example.test')).toBeNull();
        expect(screen.getByText('unknown@example.org')).toBeInTheDocument();
        expect(screen.getAllByRole('link')).toHaveLength(2);
        await expectNoAxeViolations(container);
    });
});
