import {cleanup,fireEvent,render,screen,within} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {LinkedAccountList} from './LinkedAccountList.tsx';
import {installPointerEvent} from '../../components/accounts/pointerEventPolyfill.test-util.ts';
installPointerEvent();
import type {LinkedMailboxList} from '../../api/linkedAccounts.ts';
vi.mock('../../i18n/index.ts',()=>({t:(key:string,args?:Record<string,string|number>)=>args?.address?`${key}:${args.address}`:args?.name!==undefined?`${key}:${args.name}${args.position!==undefined?`@${args.position}/${args.total}`:''}`:key,isolate:(value:string)=>value}));
afterEach(cleanup);
const value:LinkedMailboxList={primary:{accountId:'1',address:'primary@example.org',storageAuthority:'msa1_primary_verified'},accounts:[{id:'child',accountId:'1',address:'child@example.org',displayName:'Child',storageAuthority:'msa1_child_verified',status:'ready',revision:'v1',expiresAt:123}],policy:{enabled:true,maxAccounts:4,supportedAuthentication:['password']}};
function mount(data=value){
  const actions={onAdd:vi.fn(),onUnlink:vi.fn(),onSetFirst:vi.fn(),onMove:vi.fn(),onRename:vi.fn(async()=>true),onRenew:vi.fn()};
  render(()=><LinkedAccountList value={data} busy={false} {...actions}/>);
  return actions;
}
const row=(key:string)=>document.querySelector<HTMLElement>(`[data-linked-row][data-key="${key}"]`)!;
const flush=async()=>{for(let i=0;i<4;i++)await Promise.resolve();};

describe('Settings › Linked mailboxes (the management table, t48 k)',()=>{
  it('rows: slim grip, avatar, nickname over address, primary star and icon actions with tooltips and labels',()=>{
    mount();
    const child=row('child');
    expect(child.querySelector('[data-linked-grip]')).toHaveAttribute('data-tooltip','linked-accounts-drag');
    expect(child.querySelector('[data-linked-grip]')).toHaveAccessibleName('linked-accounts-move-handle:Child');
    expect(child.querySelector('.person-avatar')).not.toBeNull();
    expect(child.querySelector('[data-linked-name]')).toHaveTextContent('Child');
    expect(child).toHaveTextContent('child@example.org');
    const buttons=[...child.querySelectorAll<HTMLButtonElement>('[data-linked-actions] button')];
    expect(buttons.map(b=>Object.keys(b.dataset).find(k=>k.startsWith('linked')))).toEqual(['linkedSetFirst','linkedRename','linkedUnlink']);
    for(const button of buttons){
      expect(button.textContent?.trim()).toBe('');
      expect(button.getAttribute('data-tooltip')).toBeTruthy();
      expect(button.getAttribute('aria-label')).toContain('child@example.org');
    }
    // No status text: no "Sign-in account", no "Primary", no expired line.
    expect(document.body.textContent).not.toMatch(/linked-accounts-(primary|first|reauth)\b/);
    // The sign-in account cannot be unlinked.
    expect(row('main').querySelector('[data-linked-unlink]')).toBeNull();
  });
  it('the Link a mailbox action is a single content-sized button in the header',()=>{
    const actions=mount();
    const add=screen.getByRole('button',{name:'linked-accounts-add'});
    expect(add.className).toMatch(/addButton/);
    fireEvent.click(add);
    expect(actions.onAdd).toHaveBeenCalledOnce();
  });
  it('an empty list is a friendly line with the link action',()=>{
    mount({...value,accounts:[]});
    expect(document.querySelector('[data-linked-empty]')).toHaveTextContent('linked-accounts-empty');
    expect(document.querySelector('[data-linked-row]')).toBeNull();
    expect(screen.getByRole('button',{name:'linked-accounts-add'})).not.toBeDisabled();
  });
  it('requires explicit unlink confirmation in the shared ConfirmDialog and passes the current revision row',async()=>{
    const actions=mount();
    const unlink=screen.getByRole('button',{name:'linked-accounts-unlink-for:Child (child@example.org)'});
    expect(unlink).toHaveClass('btn--danger');
    fireEvent.click(unlink);
    expect(actions.onUnlink).not.toHaveBeenCalled();
    const dialog=screen.getByRole('alertdialog');
    expect(dialog).toHaveTextContent('linked-accounts-unlink-title:child@example.org');
    expect(dialog).toHaveTextContent('linked-accounts-unlink-help');
    fireEvent.click(within(dialog).getByRole('button',{name:'linked-accounts-unlink-confirm'}));
    await flush();
    expect(actions.onUnlink).toHaveBeenCalledExactlyOnceWith(value.accounts[0]);
  });
  it('policy off disables linking and the primary choice but keeps unlink',()=>{
    mount({...value,policy:{...value.policy,enabled:false}});
    expect(screen.getByRole('button',{name:'linked-accounts-add'})).toBeDisabled();
    expect(screen.getByRole('button',{name:'linked-accounts-set-first:child@example.org'})).toBeDisabled();
    expect(screen.getByRole('button',{name:/linked-accounts-unlink-for/})).not.toBeDisabled();
  });
  it('shows the quota rather than offering an unusable add action',()=>{
    mount({...value,policy:{...value.policy,maxAccounts:1}});
    expect(screen.getByText('linked-accounts-limit')).toBeVisible();
    expect(screen.getByRole('button',{name:'linked-accounts-add'})).toBeDisabled();
  });
  it('primary is only the filled star; the others offer it',()=>{
    const actions=mount();
    const star=screen.getByRole('button',{name:'linked-accounts-primary-mailbox-for:primary@example.org'});
    expect(star).toHaveAttribute('aria-pressed','true');
    expect(star).toHaveAttribute('data-tooltip','linked-accounts-primary-mailbox');
    fireEvent.click(star);
    expect(actions.onSetFirst).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-set-first:child@example.org'}));
    expect(actions.onSetFirst).toHaveBeenCalledExactlyOnceWith('child');
  });
  it('moves the mark to the child and lets the sign-in account take it back',()=>{
    const actions=mount({...value,primaryMailbox:'child'});
    expect(screen.getByRole('button',{name:'linked-accounts-primary-mailbox-for:child@example.org'})).toHaveAttribute('aria-pressed','true');
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-set-first:primary@example.org'}));
    expect(actions.onSetFirst).toHaveBeenCalledExactlyOnceWith(null);
  });
  it('an expired link shows a status icon and Renew access in place, with no status line or unlink advice (t48 h/k)',()=>{
    const actions=mount({...value,accounts:[{...value.accounts[0]!,status:'reauthRequired',renewal:'signIn',renewalReason:'credentialRejected'}]});
    const status=row('child').querySelector('[data-linked-status]')!;
    expect(status).toHaveAttribute('aria-label','linked-accounts-expired');
    expect(status).toHaveAttribute('data-tooltip','linked-accounts-expired');
    const renew=screen.getByRole('button',{name:'linked-accounts-renew-for:Child (child@example.org)'});
    expect(renew).toHaveAttribute('data-tooltip','linked-accounts-renew-tooltip');
    expect(renew.className).toMatch(/iconWarning/);
    expect(screen.queryByText('linked-accounts-reauth')).toBeNull();
    // Cannot become the primary while it cannot open.
    expect(screen.getByRole('button',{name:'linked-accounts-set-first:child@example.org'})).toBeDisabled();
    fireEvent.click(renew);
    expect(actions.onRenew).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({id:'child'}));
  });
  it('a ready link has no status icon or Renew access',()=>{
    mount();
    expect(document.querySelector('[data-linked-status]')).toBeNull();
    expect(screen.queryByRole('button',{name:/linked-accounts-renew-for/})).toBeNull();
  });
  it('renames inline from the Rename icon: Enter saves, Esc cancels, bad text is refused',async()=>{
    const actions=mount({...value,layout:{revision:'r1',order:['child','main'],nicknames:{child:'Work'}}});
    expect([...document.querySelectorAll<HTMLElement>('[data-linked-row]')].map(r=>r.dataset['key'])).toEqual(['child','main']);
    const rename=within(row('child')).getByRole('button',{name:'linked-accounts-rename:Work (child@example.org)'});
    expect(rename).toHaveAttribute('data-tooltip','linked-accounts-rename-short');
    fireEvent.click(rename);
    let input=row('child').querySelector<HTMLInputElement>('[data-linked-nickname]')!;
    expect(input.value).toBe('Work');
    fireEvent.keyDown(input,{key:'Escape'});
    expect(row('child').querySelector('[data-linked-nickname]')).toBeNull();
    expect(actions.onRename).not.toHaveBeenCalled();
    fireEvent.click(within(row('child')).getByRole('button',{name:/linked-accounts-rename/}));
    input=row('child').querySelector<HTMLInputElement>('[data-linked-nickname]')!;
    input.value='Bad‮name';
    fireEvent.keyDown(input,{key:'Enter'});
    await flush();
    expect(actions.onRename).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('linked-accounts-nickname-invalid');
    input.value='  Office ';
    fireEvent.keyDown(input,{key:'Enter'});
    await flush();
    expect(actions.onRename).toHaveBeenCalledExactlyOnceWith('child','Office');
  });
  it('drags a row by its grip to a new slot, with a drop line while moving',()=>{
    const second={...value.accounts[0]!,id:'second',address:'second@example.org',displayName:'Second'};
    const actions=mount({...value,accounts:[value.accounts[0]!,second]});
    const rows=[...document.querySelectorAll<HTMLElement>('[data-linked-row]')];
    expect(rows.map(r=>r.dataset['key'])).toEqual(['main','child','second']);
    rows.forEach((r,i)=>{r.getBoundingClientRect=()=>({top:i*50,bottom:i*50+50,height:50,left:0,right:300,width:300,x:0,y:i*50,toJSON(){}}) as DOMRect;});
    const grip=rows[0]!.querySelector<HTMLElement>('[data-linked-grip]')!;
    fireEvent.pointerDown(grip,{pointerId:1,button:0,pointerType:'mouse',clientX:5,clientY:25});
    fireEvent.pointerMove(document,{pointerId:1,clientX:5,clientY:140});
    expect(rows[0]).toHaveAttribute('data-dragging','true');
    expect(rows[2]).toHaveAttribute('data-drop','after');
    fireEvent.pointerUp(document,{pointerId:1,clientX:5,clientY:140});
    expect(actions.onMove).toHaveBeenCalledExactlyOnceWith(0,3);
    expect(screen.getByRole('status',{name:''}).textContent??'').toContain('linked-accounts-moved');
  });
  it('a long press anywhere on a row reorders on touch',()=>{
    vi.useFakeTimers();
    try{
      const second={...value.accounts[0]!,id:'second',address:'second@example.org',displayName:'Second'};
      const actions=mount({...value,accounts:[value.accounts[0]!,second]});
      const rows=[...document.querySelectorAll<HTMLElement>('[data-linked-row]')];
      rows.forEach((r,i)=>{r.getBoundingClientRect=()=>({top:i*50,bottom:i*50+50,height:50,left:0,right:300,width:300,x:0,y:i*50,toJSON(){}}) as DOMRect;});
      const name=rows[0]!.querySelector<HTMLElement>('[data-linked-name]')!;
      fireEvent.pointerDown(name,{pointerId:5,button:0,pointerType:'touch',clientX:100,clientY:25});
      vi.advanceTimersByTime(400);
      fireEvent.pointerMove(document,{pointerId:5,clientX:100,clientY:140});
      fireEvent.pointerUp(document,{pointerId:5,clientX:100,clientY:140});
      expect(actions.onMove).toHaveBeenCalledExactlyOnceWith(0,3);
    }finally{vi.useRealTimers();}
  });
  it('Alt+Arrow on the grip is the keyboard fallback',()=>{
    const actions=mount();
    const grips=screen.getAllByRole('button',{name:/^linked-accounts-move-handle/});
    fireEvent.keyDown(grips[0]!,{key:'ArrowUp',altKey:true});
    expect(actions.onMove).not.toHaveBeenCalled();
    fireEvent.keyDown(grips[0]!,{key:'ArrowDown',altKey:true});
    expect(actions.onMove).toHaveBeenCalledExactlyOnceWith(0,2);
    fireEvent.keyDown(grips[1]!,{key:'ArrowUp',altKey:true});
    expect(actions.onMove).toHaveBeenLastCalledWith(1,0);
  });
});
