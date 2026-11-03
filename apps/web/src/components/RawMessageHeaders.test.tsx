import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {fetchMailHeaders} from '../api/mailHeaders.ts';
import {RawMessageHeaders} from './RawMessageHeaders.tsx';

vi.mock('../api/mailHeaders.ts',async original=>({...await original<typeof import('../api/mailHeaders.ts')>(),fetchMailHeaders:vi.fn()}));
beforeEach(()=>{vi.mocked(fetchMailHeaders).mockReset();});
afterEach(()=>{vi.unstubAllGlobals();vi.useRealTimers();});
const headerText=()=>within(screen.getByRole('dialog',{name:'Raw message headers'})).getByLabelText('Raw message headers');

describe('raw message headers dialog',()=>{
  it('shows folded headers as literal text, and copies the exact original header block',async()=>{
    const raw='From: Sender <sender@example.org>\r\nReceived: first hop\r\n\tsecond line\r\nX-Example: <img src=x onerror=evil()>';
    vi.mocked(fetchMailHeaders).mockResolvedValue(raw);
    const writeText=vi.fn(async()=>{});vi.stubGlobal('navigator',{clipboard:{writeText}});
    const close=vi.fn();render(()=><RawMessageHeaders url="/mail/raw" scope="account:message" onClose={close}/>);
    const dialog=await screen.findByRole('dialog',{name:'Raw message headers'});
    await waitFor(()=>expect(within(dialog).getByLabelText('Raw message headers')).toHaveTextContent('Received: first hop'));
    expect(within(dialog).getByLabelText('Raw message headers').textContent).toBe(raw);
    expect(dialog.querySelector('img,script')).toBeNull();
    fireEvent.click(within(dialog).getByRole('button',{name:'Copy headers'}));
    await screen.findByText('Headers copied.');expect(writeText).toHaveBeenCalledWith(raw);
    fireEvent.keyDown(dialog,{key:'Escape'});expect(close).toHaveBeenCalledOnce();
  });

  it('allows retry after a fetch failure',async()=>{
    vi.mocked(fetchMailHeaders).mockRejectedValueOnce(new Error('offline')).mockResolvedValueOnce('Subject: Ready');
    render(()=><RawMessageHeaders url="/mail/raw" scope="account:message" onClose={()=>{}}/>);
    expect(await screen.findByRole('alert')).toHaveTextContent('Message headers could not be loaded.');
    fireEvent.click(screen.getByRole('button',{name:'Retry'}));
    await waitFor(()=>expect(headerText()).toHaveTextContent('Subject: Ready'));
    expect(fetchMailHeaders).toHaveBeenCalledTimes(2);
  });

  it('cancels a previous account/message request and never shows its late result',async()=>{
    const pending=new Map<string,{resolve:(raw:string)=>void;signal:AbortSignal}>();
    vi.mocked(fetchMailHeaders).mockImplementation((url,signal)=>new Promise(resolve=>pending.set(url,{resolve,signal})));
    const [message,setMessage]=createSignal('old');
    const mounted=render(()=><RawMessageHeaders url={`/mail/${message()}`} scope={`account:${message()}`} onClose={()=>{}}/>);
    await waitFor(()=>expect(pending.has('/mail/old')).toBe(true));
    setMessage('new');expect(pending.get('/mail/old')!.signal.aborted).toBe(true);
    pending.get('/mail/new')!.resolve('Subject: New');
    await waitFor(()=>expect(headerText()).toHaveTextContent('Subject: New'));
    pending.get('/mail/old')!.resolve('Subject: Old private headers');await Promise.resolve();
    expect(screen.queryByText('Subject: Old private headers')).toBeNull();
    mounted.unmount();expect(pending.get('/mail/new')!.signal.aborted).toBe(true);
  });

  it('times out a hanging request and offers retry',async()=>{
    vi.useFakeTimers();let signal:AbortSignal|undefined;
    vi.mocked(fetchMailHeaders).mockImplementation((_url,value)=>{signal=value;return new Promise(()=>{});});
    render(()=><RawMessageHeaders url="/mail/raw" scope="account:message" onClose={()=>{}}/>);
    await vi.advanceTimersByTimeAsync(10000);
    expect(signal?.aborted).toBe(true);expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.getByRole('button',{name:'Retry'})).toBeInTheDocument();
  });
});
