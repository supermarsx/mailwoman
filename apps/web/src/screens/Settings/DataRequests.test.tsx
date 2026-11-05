import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { DataRequests } from './DataRequests.tsx';
import * as api from '../../api/dataRequests.ts';
import type { DataRequest, DataRequestPolicy } from '../../api/dataRequests.ts';

vi.mock('../../api/dataRequests.ts', async importOriginal => ({
  ...await importOriginal<typeof import('../../api/dataRequests.ts')>(),
  loadOverview: vi.fn(), listRequests: vi.fn(), submitRequest: vi.fn(), confirmRequest: vi.fn(),
  cancelRequest: vi.fn(), downloadLink: vi.fn(), loadRequest: vi.fn(),
}));

const policy = (over: Partial<DataRequestPolicy> = {}): DataRequestPolicy => ({
  enabled: true,
  types: { access: true, portability: true, rectification: true, erasure: true, restriction: false, objection: true, other: true },
  reauthRequired: true, confirmationRequired: true, identityVerificationRequired: false,
  approval: { export: 'auto', erasure: 'admin', other: 'admin' }, graceDays: 14, slaDays: 30, mailAllowed: true, mailFormat: 'mbox',
  singleUseDownload: true, linkMinutes: 60, exportHours: 24,
  contact: { name: 'Data protection team', email: 'dpo@example.test', url: null }, text: { intro: {} }, ...over,
});

const request = (over: Partial<DataRequest> = {}): DataRequest => ({
  id: 'r1', kind: 'access', status: 'awaiting_confirmation', createdAt: 1_700_000_000, updatedAt: 1_700_000_000, dueAt: 1_702_592_000,
  details: '', requestedBy: 'self', needsConfirmation: true, canCancel: true, graceUntil: null, decisionReason: null,
  extensionDays: 0, extensionReason: null, export: null, downloadAvailable: false, downloads: 0, completion: null, dueState: 'ok', ...over,
});

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.loadOverview).mockResolvedValue({ policy: policy() });
  vi.mocked(api.listRequests).mockResolvedValue([]);
});

describe('data requests (Settings)', () => {
  it('offers only the enabled request types and shows the contact', async () => {
    render(() => <DataRequests />);
    const select = await screen.findByLabelText('Request type');
    const options = Array.from(select.querySelectorAll('option')).map(o => o.textContent);
    expect(options).toContain('Erasure');
    expect(options).not.toContain('Restriction');
    expect(screen.getByTestId('dr-contact')).toHaveTextContent('Data protection team');
    expect(screen.getByText('You have not filed any requests.')).toBeInTheDocument();
  });

  it('asks the person to tick before an erasure can be submitted, and submits it', async () => {
    vi.mocked(api.submitRequest).mockResolvedValue(request({ kind: 'erasure', confirmationSent: true }));
    render(() => <DataRequests />);
    fireEvent.change(await screen.findByLabelText('Request type'), { target: { value: 'erasure' } });
    expect(screen.getByText(/waiting period of 14 day/)).toBeInTheDocument();
    const submit = screen.getByRole('button', { name: 'Submit request' });
    expect(submit).toBeDisabled();
    fireEvent.click(screen.getByLabelText('I understand that my data will be erased.'));
    expect(submit).toBeEnabled();
    fireEvent.click(submit);
    await waitFor(() => expect(api.submitRequest).toHaveBeenCalledWith(expect.objectContaining({ kind: 'erasure', includeMail: false })));
    expect(await screen.findByText(/confirmation code was put in your inbox/)).toBeInTheDocument();
  });

  it('only shows the mail option for export requests and passes it on', async () => {
    vi.mocked(api.submitRequest).mockResolvedValue(request({ confirmationSent: false }));
    render(() => <DataRequests />);
    const mail = await screen.findByLabelText('Include my messages stored on this server');
    fireEvent.click(mail);
    fireEvent.click(screen.getByRole('button', { name: 'Submit request' }));
    await waitFor(() => expect(api.submitRequest).toHaveBeenCalledWith(expect.objectContaining({ kind: 'access', includeMail: true })));
    fireEvent.change(screen.getByLabelText('Request type'), { target: { value: 'objection' } });
    expect(screen.queryByLabelText('Include my messages stored on this server')).toBeNull();
  });

  it('confirms with a code, cancels a scheduled erasure and downloads a ready export', async () => {
    vi.mocked(api.listRequests).mockResolvedValue([
      request(),
      request({ id: 'r2', kind: 'erasure', status: 'scheduled', needsConfirmation: false, graceUntil: 1_701_000_000 }),
      request({ id: 'r3', status: 'ready', needsConfirmation: false, downloadAvailable: true, export: { bytes: 4096, expiresAt: 1_701_000_000, complete: true } }),
    ]);
    vi.mocked(api.confirmRequest).mockResolvedValue(request({ needsConfirmation: false }));
    vi.mocked(api.cancelRequest).mockResolvedValue(request({ id: 'r2', status: 'cancelled' }));
    vi.mocked(api.downloadLink).mockResolvedValue({ url: '/api/privacy/requests/r3/download?token=t', expiresAt: 1, singleUse: true });
    render(() => <DataRequests />);
    const first = await screen.findByTestId('dr-r1');
    fireEvent.input(first.querySelector('input')!, { target: { value: 'ABCD2345' } });
    fireEvent.click(first.querySelector('button')!);
    await waitFor(() => expect(api.confirmRequest).toHaveBeenCalledWith('r1', 'ABCD2345'));
    const second = screen.getByTestId('dr-r2');
    expect(second).toHaveTextContent('You can cancel until then');
    fireEvent.click(Array.from(second.querySelectorAll('button')).find(b => b.textContent === 'Cancel erasure')!);
    await waitFor(() => expect(api.cancelRequest).toHaveBeenCalledWith('r2'));
    const third = screen.getByTestId('dr-r3');
    expect(third).toHaveTextContent('4 KB');
    fireEvent.click(Array.from(third.querySelectorAll('button')).find(b => b.textContent === 'Download')!);
    await waitFor(() => expect(api.downloadLink).toHaveBeenCalledWith('r3'));
  });

  it('says so when the feature is switched off', async () => {
    vi.mocked(api.loadOverview).mockResolvedValue({ policy: policy({ enabled: false }) });
    render(() => <DataRequests />);
    expect(await screen.findByText('Data requests are switched off on this server.')).toBeInTheDocument();
  });
});
