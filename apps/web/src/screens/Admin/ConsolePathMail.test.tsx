import {describe,it,expect,vi} from 'vitest';
import {fireEvent,screen,within,waitFor} from '@solidjs/testing-library';
import {ConsolePathMail} from './ConsolePathMail.tsx';
import {renderInShell} from './shell/testkit.tsx';
import {AdminProblemError} from './shell/adminApi.ts';
import {validRecipient,type ConsolePathMailApi,type ConsolePathMailStatus} from '../../api/admin/consolePath.ts';

const status=(over:Partial<ConsolePathMailStatus>={}):ConsolePathMailStatus=>({configured:true,reason:null,suggestions:[{address:'root@example.org',kind:'signIn'},{address:'ops@example.org',kind:'operator'}],perAdminPerHour:3,...over});
const api=(data:ConsolePathMailStatus,send:ConsolePathMailApi['send']=vi.fn(async(recipient:string)=>({data:{queued:true,recipient},etag:null,status:202}))):ConsolePathMailApi=>({status:async()=>({data,etag:null,status:200}),send});

describe('console address by email (t58)',()=>{
  it('accepts one plain address only',()=>{
    for(const ok of ['ops@example.org',' a.b+c@mail.example.org '])expect(validRecipient(ok),ok).toBe(true);
    for(const bad of ['','root','a@b','a@example.org, b@example.org','Ops <ops@example.org>','a b@example.org','a@[127.0.0.1]','a@-x.org'])expect(validRecipient(bad),bad).toBe(false);
  });

  it('is off with an explanation while outbound email is not set up',async()=>{
    renderInShell(()=><ConsolePathMail pending={false} api={api(status({configured:false,reason:'disabled'}))}/>);
    const button=await screen.findByRole('button',{name:'Send by email'});
    await waitFor(()=>expect(button).toBeDisabled());
    expect(button).toHaveAccessibleDescription(/Set it up in Compliance > Email notifications/);
  });

  it('sends to the preselected own address in the console language and confirms with a toast',async()=>{
    const send=vi.fn(async(recipient:string)=>({data:{queued:true,recipient},etag:null,status:202}));
    renderInShell(()=><ConsolePathMail pending={false} api={api(status(),send)}/>);
    const button=await screen.findByRole('button',{name:'Send by email'});
    await waitFor(()=>expect(button).toBeEnabled());
    fireEvent.click(button);
    const dialog=await screen.findByRole('alertdialog');
    expect(within(dialog).getByText(/It contains no password or code/)).toBeVisible();
    expect(within(dialog).getByLabelText('root@example.org (your sign-in name)')).toBeChecked();
    expect(within(dialog).getByText('Each administrator can send 3 of these messages per hour.')).toBeVisible();
    fireEvent.click(within(dialog).getByRole('button',{name:'Send'}));
    await waitFor(()=>expect(send).toHaveBeenCalledWith('root@example.org','en'));
    await waitFor(()=>expect(screen.queryByRole('alertdialog')).toBeNull());
    expect(await screen.findByText(/Message queued for root@example.org/)).toBeVisible();
  });

  it('checks a typed address before sending and shows the server refusal in the dialog',async()=>{
    const send=vi.fn(async()=>{throw new AdminProblemError(429,{code:'rateLimited',detail:'x',retryAfterSecs:60},null);});
    renderInShell(()=><ConsolePathMail pending api={api(status(),send)}/>);
    const button=await screen.findByRole('button',{name:'Send by email'});
    await waitFor(()=>expect(button).toBeEnabled());
    fireEvent.click(button);
    const dialog=await screen.findByRole('alertdialog');
    expect(within(dialog).getByText(/address that applies after the next restart/)).toBeVisible();
    fireEvent.click(within(dialog).getByLabelText('Another address'));
    const field=within(dialog).getByLabelText('Email address');
    fireEvent.input(field,{target:{value:'a@example.org, b@example.org'}});
    expect(field).toHaveAttribute('aria-invalid','true');
    fireEvent.click(within(dialog).getByRole('button',{name:'Send'}));
    expect(await within(dialog).findByText('Enter one email address, without a name or a second address.')).toBeVisible();
    expect(send).not.toHaveBeenCalled();
    fireEvent.input(field,{target:{value:'other@example.net'}});
    fireEvent.click(within(dialog).getByRole('button',{name:'Send'}));
    await waitFor(()=>expect(send).toHaveBeenCalledWith('other@example.net','en'));
    expect(await within(dialog).findByText(/Too many console address emails/)).toBeVisible();
    expect(screen.getByRole('alertdialog')).toBeVisible();
  });

  it('offers only the typed address when there is nothing to suggest, and Escape closes',async()=>{
    renderInShell(()=><ConsolePathMail pending={false} api={api(status({suggestions:[]}))}/>);
    const button=await screen.findByRole('button',{name:'Send by email'});
    await waitFor(()=>expect(button).toBeEnabled());
    fireEvent.click(button);
    const dialog=await screen.findByRole('alertdialog');
    expect(within(dialog).queryByRole('radio')).toBeNull();
    expect(within(dialog).getByLabelText('Email address')).toBeVisible();
    fireEvent.keyDown(dialog,{key:'Escape'});
    await waitFor(()=>expect(screen.queryByRole('alertdialog')).toBeNull());
  });
});
