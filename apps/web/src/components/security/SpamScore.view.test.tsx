import { afterEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
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
  expect(screen.queryByRole('dialog')).toBeNull();
});
it('keeps the breakdown out of the reader until the chip opens it in the shared popover',async()=>{
  vi.mocked(fetchMailHeaders).mockResolvedValue([
    'X-Spam-Status: Yes, score=6.2 required=5.0 tests=BAYES_99,DKIM_VALID autolearn=no',
    'X-Spam-Report: *  6.3 BAYES_99 BODY: Bayes spam probability is 99 to 100%\r\n * -0.1 DKIM_VALID Message has at least one valid DKIM or DK signature',
    'X-Spamd-Result: default: False [-0.50 / 15.00]; SPF_ALLOW(-0.30)[+ip4:192.0.2.1]; DKIM_ALLOW(-0.20)[example.test:s=sel]',
  ].join('\r\n'));
  render(()=><SpamScore url="/mail/raw"/>);
  const chip=await screen.findByRole('button',{name:/SpamAssassin/});
  expect(chip).toHaveAttribute('aria-expanded','false');
  expect(chip).toHaveAttribute('aria-haspopup','dialog');
  expect(chip.className).toContain('spam-score__chip--spam');
  expect(screen.getByRole('button',{name:/Rspamd/}).className).toContain('spam-score__chip--ham');
  expect(screen.queryByRole('dialog')).toBeNull();
  fireEvent.click(chip);
  const dialog=await screen.findByRole('dialog');
  expect(chip).toHaveAttribute('aria-expanded','true');
  expect(dialog).toHaveAccessibleName(/SpamAssassin/);
  await waitFor(()=>expect(dialog).toHaveFocus());
  const rows=within(dialog).getAllByRole('row').map(row=>row.textContent);
  expect(rows[1]).toContain('BAYES_99');expect(rows[1]).toContain('+6.3');
  expect(rows[2]).toContain('DKIM_VALID');expect(rows[2]).toContain('−0.1');
  expect(rows.at(-1)).toContain('6.2');
  expect(within(dialog).getByText(/not independently verified/)).toBeVisible();
  fireEvent.keyDown(dialog,{key:'Escape'});
  await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());
  expect(chip).toHaveFocus();
  fireEvent.click(screen.getByRole('button',{name:/Rspamd/}));
  const rspamd=await screen.findByRole('dialog');
  expect(within(rspamd).getAllByRole('row').map(row=>row.querySelector('code')?.textContent).filter(Boolean)).toEqual(['DKIM_ALLOW','SPF_ALLOW']);
  expect(within(rspamd).getByText('example.test:s=sel')).toBeVisible();
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
