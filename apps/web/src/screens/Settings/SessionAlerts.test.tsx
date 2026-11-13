import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { SessionAlerts } from './SessionAlerts.tsx';
import { SessionAlertBanner } from './SessionAlertBanner.tsx';

const fixture = vi.hoisted(() => ({
  locked: false,
  unread: 1,
  put: vi.fn(async (_k: string, _v: unknown) => ({})),
  ack: vi.fn(async (_t: unknown) => undefined),
  context:undefined as import('./sessionAlertsModel.ts').SessionNoticeContext|undefined,
}));
vi.mock('../../api/settings.ts', () => ({
  fetchSettings: async () => ({
    'security.alerts.countryChange': { value: true, source: 'default', locked: fixture.locked },
    'security.alerts.channels': { value: 'both', source: 'default', locked: false },
    'security.alerts.sensitivity': { value: 'normal', source: 'default', locked: false },
  }),
  putSetting: (k: string, v: unknown) => fixture.put(k, v),
  deleteSetting: async () => ({}),
}));
vi.mock('./sessionAlertsModel.ts', async (original) => ({
  ...(await original<typeof import('./sessionAlertsModel.ts')>()),
  fetchSessionAlerts: async () => ({
    unread: fixture.unread,
    notices: [{ id: 'n1', ts: 1_800_000_000, kind: 'countryChange', fromCountry: 'PT', toCountry: 'BR', action: 'alerted', session: 'ab12cd', acknowledged: false,context:fixture.context }],
  }),
  acknowledgeSessionAlerts: (t: unknown) => fixture.ack(t),
}));

afterEach(() => {
  cleanup();
  fixture.locked = false;
  fixture.unread = 1;
  fixture.put.mockClear();
  fixture.ack.mockClear();
  fixture.context=undefined;
});

it('shows a notice with countries, session id and the accuracy caveat, and dismisses it', async () => {
  render(() => <SessionAlerts service={{ revokeOtherSessions: vi.fn() } as never} />);
  expect((await screen.findAllByText(/PT to BR/)).length).toBe(2);
  expect(screen.getByText(/ab12cd/)).toBeInTheDocument();
  expect(screen.getAllByText(/Country detection may be inaccurate/).length).toBeGreaterThan(0);
  fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
  await waitFor(() => expect(fixture.ack).toHaveBeenCalledWith({ id: 'n1' }));
});
it('shows event-time device evidence with internal location and guarded technical details',async()=>{
 fixture.context={ip:'192.168.2.3',country:null,deviceType:'desktop',browser:'chrome',browserMajor:142,browserEngine:'Blink',os:'windows',clientType:'browser',recognized:false,fingerprint:'1234567890abcdef',userAgent:'Reported browser agent'};
 render(()=><SessionAlerts service={{} as never}/>);
 expect(await screen.findByText(/192\.168\.2\.3/)).toBeInTheDocument();expect(screen.getByText(/Internal network/)).toBeInTheDocument();expect(screen.getByText(/Chrome 142/)).toBeInTheDocument();expect(screen.getByText(/Unrecognized device/)).toBeInTheDocument();
 const details=screen.getByText('Technical details').closest('details')!;expect(details.open).toBe(false);fireEvent.click(screen.getByText('Technical details'));expect(screen.getByText('1234567890abcdef')).toBeInTheDocument();
});

it("'This wasn't me' signs the other sessions out and points to the password", async () => {
  const revoke = vi.fn(async () => undefined);
  render(() => <SessionAlerts service={{ revokeOtherSessions: revoke } as never} />);
  fireEvent.click(await screen.findByRole('button', { name: "This wasn't me" }));
  expect(await screen.findByTestId('session-alerts-revoked')).toBeInTheDocument();
  expect(revoke).toHaveBeenCalledOnce();
});

it('writes preferences and disables a locked control', async () => {
  fixture.locked = true;
  render(() => <SessionAlerts service={{} as never} />);
  const toggle = await screen.findByLabelText(/Warn me when a session is used from a new country/);
  expect(toggle).toBeDisabled();
  expect(screen.getByTestId('setting-lock-note')).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText('Sensitivity'), { target: { value: 'high' } });
  await waitFor(() => expect(fixture.put).toHaveBeenCalledWith('security.alerts.sensitivity', 'high'));
});

it('banner appears only with unread notices', async () => {
  const onReview = vi.fn();
  render(() => <SessionAlertBanner onReview={onReview} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Review' }));
  expect(onReview).toHaveBeenCalled();
  cleanup();
  fixture.unread = 0;
  render(() => <SessionAlertBanner />);
  await new Promise((r) => setTimeout(r, 20));
  expect(screen.queryByTestId('session-alert-banner')).toBeNull();
});
