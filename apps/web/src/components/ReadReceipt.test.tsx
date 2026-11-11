import {it,expect,vi,afterEach} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {ReadReceipt,type ReadReceiptApi} from './ReadReceipt.tsx';
const service=():ReadReceiptApi=>({load:vi.fn(async(accountId:string,emailId:string)=>({accountId,emailId,requestedBy:'sender@example.org',handled:false,action:'automatic' as const})),send:vi.fn(async()=>{})});
afterEach(()=>vi.useRealTimers());
it('never loads a request before display and never automatically answers a preview',async()=>{
 const api=service();const [displayed,setDisplayed]=createSignal(false);
 render(()=><ReadReceipt accountId="a" emailId="m" active displayed={displayed()} fullView={false} api={api}/>);
 expect(api.load).not.toHaveBeenCalled();setDisplayed(true);await screen.findByRole('button',{name:'Send read receipt'});
 await new Promise(resolve=>setTimeout(resolve,1600));expect(api.send).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Send read receipt'}));await waitFor(()=>expect(api.send).toHaveBeenCalledWith('a','m',false,expect.any(AbortSignal)));
});
it('requires an active displayed full view before the automatic timer starts',async()=>{
 const api=service();const [active,setActive]=createSignal(false);
 render(()=><ReadReceipt accountId="a" emailId="m" active={active()} displayed fullView api={api}/>);
 await waitFor(()=>expect(api.load).toHaveBeenCalled());expect(api.send).not.toHaveBeenCalled();setActive(true);
 await waitFor(()=>expect(api.send).toHaveBeenCalledWith('a','m',true,expect.any(AbortSignal)),{timeout:2200});
 expect(api.send).toHaveBeenCalledTimes(1);
});
it('does not retry an uncertain result when the same body is redisplayed',async()=>{
 const api=service();vi.mocked(api.send).mockRejectedValue(new Error('connection lost'));const [displayed,setDisplayed]=createSignal(true);
 render(()=><ReadReceipt accountId="a" emailId="m" active displayed={displayed()} fullView={false} api={api}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Send read receipt'}));await screen.findByRole('status');setDisplayed(false);setDisplayed(true);
 await waitFor(()=>expect(api.load).toHaveBeenCalledTimes(2));expect(api.send).toHaveBeenCalledTimes(1);expect(screen.queryByRole('button',{name:'Send read receipt'})).not.toBeInTheDocument();
});
it('a receipt being sent is not aborted when the decision reloads (t93)',async()=>{
 const api=service();let finish:()=>void=()=>undefined;let sent:AbortSignal|undefined;
 vi.mocked(api.send).mockImplementation((_a,_e,_automatic,signal)=>{sent=signal;return new Promise<void>(resolve=>{finish=resolve;});});
 const [displayed,setDisplayed]=createSignal(true);
 render(()=><ReadReceipt accountId="a" emailId="m" active displayed={displayed()} fullView={false} api={api}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Send read receipt'}));await waitFor(()=>expect(api.send).toHaveBeenCalledTimes(1));
 setDisplayed(false);setDisplayed(true);await waitFor(()=>expect(api.load).toHaveBeenCalledTimes(2));
 expect(sent?.aborted).toBe(false);finish();
 expect(api.send).toHaveBeenCalledTimes(1);
});

it('enrols an eligible sender request only when a full reader is active',async()=>{
 const api=service();
 const at=Math.floor(Date.now()/1000)+600;
 api.schedule=vi.fn(async()=>{});
 const decision={accountId:'a',emailId:'m',requestedBy:'sender@example.org',handled:false,action:'automatic' as const,scheduleAllowed:true,scheduleSender:'sender@example.org',senderScheduleAt:at};
 vi.mocked(api.load).mockResolvedValueOnce(decision).mockResolvedValue({...decision,scheduledAt:at});
 const [full,setFull]=createSignal(false);
 render(()=><ReadReceipt accountId="a" emailId="m" active displayed fullView={full()} api={api}/>);
 await screen.findByRole('button',{name:'Schedule receipt'});
 expect(api.schedule).not.toHaveBeenCalled();
 setFull(true);
 await screen.findByRole('button',{name:'Cancel scheduled receipt'});
 expect(api.schedule).toHaveBeenCalledWith('m','enrol',null,expect.any(AbortSignal));
 expect(api.send).not.toHaveBeenCalled();
});
