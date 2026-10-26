import { afterEach, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { createSignal, Suspense } from 'solid-js';
import { deliveryReport, fetchDeliveryReport, type DeliveryReport } from '../../api/dsn.ts';
import { loadCatalog } from '../../i18n/index.ts';
import { DeliveryStatus } from './DeliveryStatus.tsx';
vi.mock('../../api/dsn.ts',async original=>({...await original<typeof import('../../api/dsn.ts')>(),fetchDeliveryReport:vi.fn()}));
const report:DeliveryReport={reportingMta:'mx.example.org',envelopeId:null,arrivalDate:null,recipients:[{finalRecipient:'person@example.org',addressType:'rfc822',originalRecipient:null,action:'failed',status:'5.1.1',diagnostic:'smtp; <img src=x onerror=alert(1)>',remoteMta:null,lastAttempt:null,retryUntil:null}]};
afterEach(()=>vi.resetAllMocks());
it('renders escaped report claims without turning delivery into a read receipt',async()=>{
  await loadCatalog('mail-dsn');vi.mocked(fetchDeliveryReport).mockResolvedValue(report);
  const {container}=render(()=><DeliveryStatus accountId="a" emailId="m" enabled/>);
  expect(await screen.findByRole('heading',{name:'Delivery status report'})).toBeVisible();
  expect(screen.getByText(/not independently verified/)).toBeVisible();
  expect(screen.getByText(/<img src=x/)).toBeVisible();
  expect(container.querySelector('img')).toBeNull();
  expect(screen.getByText('Reported failure')).toBeVisible();
});
it('does not query unsupported proxy sessions or suspend the whole reader',async()=>{
  render(()=><Suspense fallback={<p>Pending page</p>}><h1>Reader</h1><DeliveryStatus accountId="a" emailId="m" enabled={false}/></Suspense>);
  expect(screen.getByRole('heading',{name:'Reader'})).toBeVisible();
  expect(screen.queryByText('Pending page')).toBeNull();
  expect(fetchDeliveryReport).not.toHaveBeenCalled();
});
it('fences an old response after switching accounts and hides failed fetches',async()=>{
  let old!:(r:DeliveryReport)=>void;
  vi.mocked(fetchDeliveryReport).mockImplementation(account=>account==='old'?new Promise(resolve=>{old=resolve;}):Promise.resolve(null));
  const [account,setAccount]=createSignal('old');
  render(()=><DeliveryStatus accountId={account()} emailId="same-id" enabled/>);
  await waitFor(()=>expect(fetchDeliveryReport).toHaveBeenCalledWith('old','same-id'));
  setAccount('new');
  await waitFor(()=>expect(fetchDeliveryReport).toHaveBeenCalledWith('new','same-id'));
  old(report);await Promise.resolve();
  expect(screen.queryByRole('heading',{name:'Delivery status report'})).toBeNull();
});
it('rejects malformed and unbounded wire projections',()=>{
  expect(deliveryReport(report)).toEqual(report);
  expect(deliveryReport({...report,recipients:Array(65).fill(report.recipients[0])})).toBeNull();
  expect(deliveryReport({...report,recipients:[{...report.recipients[0],status:'success'}]})).toBeNull();
  expect(deliveryReport({...report,reportingMta:'x'.repeat(4097)})).toBeNull();
});
