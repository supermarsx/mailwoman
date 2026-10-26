import { afterEach, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { createSignal, Suspense } from 'solid-js';
import { fetchMailHeaders } from '../../api/mailHeaders.ts';
import { SpamScore } from './SpamScore.tsx';
vi.mock('../../api/mailHeaders.ts',async original=>({...await original<typeof import('../../api/mailHeaders.ts')>(),fetchMailHeaders:vi.fn()}));
afterEach(()=>vi.resetAllMocks());
it('shows compact header-reported scores without suspending the surrounding reader',async()=>{
  let resolve!:(raw:string)=>void;
  vi.mocked(fetchMailHeaders).mockReturnValue(new Promise(done=>{resolve=done;}));
  render(()=><Suspense fallback={<p>Whole page pending</p>}><h1>Message reader</h1><SpamScore url="/mail/raw"/></Suspense>);
  expect(screen.getByRole('heading',{name:'Message reader'})).toBeVisible();
  expect(screen.queryByText('Whole page pending')).toBeNull();
  resolve('X-Spam-Status: Yes, score=6.2 required=5.0 tests=RULE_ONE');
  expect(await screen.findByText('6.2 / 5')).toBeInTheDocument();
  expect(screen.getByText(/not independently verified/)).toBeInTheDocument();
});
it('does not invent a zero score when headers cannot be retrieved',async()=>{
  vi.mocked(fetchMailHeaders).mockRejectedValue(new Error('offline'));
  render(()=><SpamScore url="/mail/raw"/>);
  expect(await screen.findByText('Spam score unavailable')).toBeVisible();
  expect(screen.queryByText('0 / 5')).toBeNull();
});

it('does not publish headers from an aborted previous message',async()=>{
  const pending=new Map<string,(raw:string)=>void>();
  vi.mocked(fetchMailHeaders).mockImplementation(url=>new Promise(resolve=>pending.set(url,resolve)));
  const received=vi.fn();const [url,setUrl]=createSignal('/mail/old');
  render(()=><SpamScore url={url()} onHeaders={received}/>);
  setUrl('/mail/new');
  pending.get('/mail/new')!('X-Spam-Score: 1');
  await waitFor(()=>expect(received).toHaveBeenCalledWith('X-Spam-Score: 1','/mail/new'));
  pending.get('/mail/old')!('List-Unsubscribe: <https://old.example>');
  await Promise.resolve();
  expect(received.mock.calls).toEqual([[null,'/mail/old'],[null,'/mail/new'],['X-Spam-Score: 1','/mail/new']]);
});
