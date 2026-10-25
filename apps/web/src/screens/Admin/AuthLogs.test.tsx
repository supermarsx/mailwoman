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
