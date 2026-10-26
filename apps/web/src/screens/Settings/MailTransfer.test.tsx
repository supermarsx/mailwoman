import {describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {MailTransfer,MailExportEditor} from './MailTransfer.tsx';
import {MailTransferError,type MailTransferApi,type MailImportResult} from '../../api/mailTransfer.ts';
function service():MailTransferApi{return {
  formats:vi.fn(async()=>({engineMode:true,available:true,maxBytes:1024,maxMessages:100})),
  import:vi.fn(async()=>({imported:1,failed:0,total:1,mailboxId:'archive'})),export:vi.fn(),
};}
const mailboxes=[{id:'archive',name:'Archive'},{id:'inbox',name:'Inbox'}];
async function choose():Promise<void>{
  fireEvent.click(await screen.findByRole('button',{name:'Import mail'}));
  fireEvent.change(await screen.findByLabelText('Destination folder'),{target:{value:'archive'}});
  fireEvent.change(screen.getByLabelText('Mail file'),{target:{files:[new File(['Subject: example\r\n\r\nbody'],'mail.eml')]}});
}
describe('dedicated mail transfer editor',()=>{
  it('passes the selected export format through the policy-protected API',async()=>{
    const api=service();vi.mocked(api.export).mockRejectedValue(new MailTransferError(403));
    render(()=><MailExportEditor accountId="a" emailId="message" api={api} onBack={async()=>{}}/>);
    fireEvent.change(await screen.findByLabelText('File format'),{target:{value:'msg'}});
    fireEvent.click(screen.getByRole('button',{name:'Download selected format'}));
    expect(await screen.findByRole('alert')).toHaveTextContent('Export is restricted');
    expect(api.export).toHaveBeenCalledWith('message','msg',expect.any(AbortSignal));
  });
  it('drops an export response after switching account',async()=>{
    const api=service();let complete!:(value:Blob)=>void;
    vi.mocked(api.export).mockImplementation(()=>new Promise(resolve=>{complete=resolve;}));
    const [account,setAccount]=createSignal('a');
    render(()=><MailExportEditor accountId={account()} emailId="message" api={api} onBack={async()=>{}}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Download selected format'}));
    const signal=vi.mocked(api.export).mock.calls[0]![2];setAccount('b');
    await waitFor(()=>expect(signal.aborted).toBe(true));complete(new Blob(['old account message']));await Promise.resolve();
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
  it('writes only after explicit folder and submit, then clears file to avoid replay',async()=>{
    const api=service();render(()=><MailTransfer accountId="a" mailboxes={mailboxes} api={api}/>);
    await choose();expect(api.import).not.toHaveBeenCalled();
    expect(screen.queryByRole('heading',{name:'Import and export mail'})).not.toBeInTheDocument();
    fireEvent.submit(screen.getByRole('button',{name:'Import into selected folder'}).closest('form')!);
    await screen.findByRole('status');expect(api.import).toHaveBeenCalledTimes(1);
    expect(api.import).toHaveBeenCalledWith('eml',expect.any(File),'archive',expect.any(AbortSignal));
    expect(screen.getByRole('button',{name:'Import into selected folder'})).toBeDisabled();
  });
  it('does not expose a form when runtime or administrator refuses import',async()=>{
    const api=service();vi.mocked(api.formats).mockRejectedValue(new MailTransferError(403));
    render(()=><MailTransfer accountId="a" mailboxes={mailboxes} api={api}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Import mail'}));
    expect(await screen.findByRole('alert')).toHaveTextContent('administrator');
    expect(screen.queryByLabelText('Mail file')).not.toBeInTheDocument();expect(api.import).not.toHaveBeenCalled();
  });
  it('keeps read-only or unconfigured connections unavailable even in engine mode',async()=>{
    const api=service();vi.mocked(api.formats).mockResolvedValue({engineMode:true,available:false,maxBytes:1024,maxMessages:100});
    render(()=><MailTransfer accountId="a" mailboxes={mailboxes} api={api}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Import mail'}));
    await waitFor(()=>expect(screen.getByRole('status')).toHaveTextContent('unavailable'));
    expect(screen.queryByLabelText('Mail file')).not.toBeInTheDocument();
  });
  it('does not retry uncertain failures',async()=>{
    const api=service();vi.mocked(api.import).mockRejectedValue(new Error('connection lost'));
    render(()=><MailTransfer accountId="a" mailboxes={mailboxes} api={api}/>);await choose();
    fireEvent.submit(screen.getByRole('button',{name:'Import into selected folder'}).closest('form')!);
    expect(await screen.findByRole('alert')).toHaveTextContent('Some messages may already be saved');
    expect(api.import).toHaveBeenCalledTimes(1);
  });
  it('aborts and drops completion when the authenticated account changes',async()=>{
    const api=service();let complete!:(value:MailImportResult)=>void;
    vi.mocked(api.import).mockImplementation(()=>new Promise(resolve=>{complete=resolve;}));
    const [account,setAccount]=createSignal('a');const refresh=vi.fn();
    render(()=><MailTransfer accountId={account()} mailboxes={mailboxes} api={api} onImported={refresh}/>);await choose();
    fireEvent.submit(screen.getByRole('button',{name:'Import into selected folder'}).closest('form')!);
    await waitFor(()=>expect(api.import).toHaveBeenCalledTimes(1));
    const signal=vi.mocked(api.import).mock.calls[0]![3];setAccount('b');
    await waitFor(()=>expect(signal.aborted).toBe(true));
    complete({imported:1,failed:0,total:1,mailboxId:'archive'});await Promise.resolve();
    expect(refresh).not.toHaveBeenCalled();expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});
