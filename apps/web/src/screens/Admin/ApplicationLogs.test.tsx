import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { ApplicationLogs, type ApplicationLogsApi } from './ApplicationLogs.tsx';
import { renderInShell } from './shell/testkit.tsx';
import { expectNoAxeViolations } from './shell/uxTestkit.ts';
describe('application logs', () => {
    it('labels process-local limits and renders redacted fields as text', async () => {
        const history = vi.fn<ApplicationLogsApi['history']>(async () => ({ kind: 'ok', etag: null, data: { events: [{ id: 1, firstSeen: 100, lastSeen: 102, repeatCount: 3, level: 'WARN', target: 'mw_test', fields: { message: '<img src=x onerror=alert(1)>', password: '[redacted]' } }], cursor: null, discarded: 2, capacity: 1000, retentionSeconds: 86400, processLocal: true } }));
        const { container } = renderInShell(() => <ApplicationLogs api={{ history }}/>);
        expect(await screen.findByText('[redacted]')).toBeInTheDocument();
        expect(screen.getByText(/cleared on restart/i)).toBeInTheDocument();
        expect(container.querySelector('img')).toBeNull();
        fireEvent.change(screen.getByLabelText('Severity'), { target: { value: 'WARN' } });
        fireEvent.click(screen.getByRole('button', { name: 'Apply filters' }));
        await vi.waitFor(() => expect(history).toHaveBeenCalledTimes(2));
        expect(history.mock.calls[1]![0].get('level')).toBe('WARN');
        await expectNoAxeViolations(container);
    });
});
