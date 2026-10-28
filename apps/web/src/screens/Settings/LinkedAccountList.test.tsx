import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {LinkedAccountList} from './LinkedAccountList.tsx';
import {installPointerEvent} from '../../components/accounts/pointerEventPolyfill.test-util.ts';
installPointerEvent();
import type {LinkedMailboxList} from '../../api/linkedAccounts.ts';
vi.mock('../../i18n/index.ts',()=>({t:(key:string,args?:Record<string,string|number>)=>args?.address?`${key}:${args.address}`:args?.name!==undefined?`${key}:${args.name}${args.position!==undefined?`@${args.position}/${args.total}`:''}`:key,isolate:(value:string)=>value}));
afterEach(cleanup);
const value:LinkedMailboxList={primary:{accountId:'1',address:'primary@example.org',storageAuthority:'msa1_primary_verified'},accounts:[{id:'child',accountId:'1',address:'child@example.org',displayName:'Child',storageAuthority:'msa1_child_verified',status:'ready',revision:'v1',expiresAt:123}],policy:{enabled:true,maxAccounts:4,supportedAuthentication:['password']}};
function mount(data=value){
  const actions={onAdd:vi.fn(),onSwitch:vi.fn(),onUnlink:vi.fn(),onSetFirst:vi.fn(),onMove:vi.fn(),onRename:vi.fn(async()=>true)};
  render(()=><LinkedAccountList value={data} activeLinkId={null} busy={false} {...actions}/>);
  return actions;
}
describe('linked mailbox management',()=>{
  it('requires explicit unlink confirmation and passes the current revision row',()=>{
    const actions=mount();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-unlink'}));
    expect(actions.onUnlink).not.toHaveBeenCalled();
    expect(screen.getByText('linked-accounts-unlink-help')).toBeVisible();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-unlink-confirm'}));
    expect(actions.onUnlink).toHaveBeenCalledExactlyOnceWith(value.accounts[0]);
  });
  it('disables linking and child switching under policy while keeping unlink available',()=>{
    mount({...value,policy:{...value.policy,enabled:false}});
    expect(screen.getByRole('button',{name:'linked-accounts-add'})).toBeDisabled();
    expect(screen.getAllByRole('button',{name:'linked-accounts-open'})[1]).toBeDisabled();
    expect(screen.getByRole('button',{name:'linked-accounts-unlink'})).not.toBeDisabled();
  });
  it('shows the quota rather than offering an unusable add action',()=>{
    mount({...value,policy:{...value.policy,maxAccounts:1}});
    expect(screen.getByText('linked-accounts-limit')).toBeVisible();
    expect(screen.getByRole('button',{name:'linked-accounts-add'})).toBeDisabled();
  });
  it('marks the mailbox that opens first and offers the others (t40)',()=>{
    const actions=mount();
    expect(screen.getByText('linked-accounts-first · linked-accounts-first-help')).toBeVisible();
    expect(screen.queryByRole('button',{name:'linked-accounts-set-first:primary@example.org'})).toBeNull();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-set-first:child@example.org'}));
    expect(actions.onSetFirst).toHaveBeenCalledExactlyOnceWith('child');
  });
  it('moves the mark to the child and lets the sign-in account take it back',()=>{
    const actions=mount({...value,primaryMailbox:'child'});
    expect(screen.queryByRole('button',{name:'linked-accounts-set-first:child@example.org'})).toBeNull();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-set-first:primary@example.org'}));
    expect(actions.onSetFirst).toHaveBeenCalledExactlyOnceWith(null);
  });
  it('does not offer an unavailable mailbox as the first one',()=>{
    mount({...value,accounts:[{...value.accounts[0]!,status:'reauthRequired'}]});
    expect(screen.getByRole('button',{name:'linked-accounts-set-first:child@example.org'})).toBeDisabled();
  });
  it('drags a row by its grip to a new slot, with a drop line while moving (t40 f)',()=>{
    const second={...value.accounts[0]!,id:'second',address:'second@example.org',displayName:'Second'};
    const actions=mount({...value,accounts:[value.accounts[0]!,second]});
    const rows=[...document.querySelectorAll<HTMLElement>('[data-linked-row]')];
    expect(rows.map(row=>row.dataset['key'])).toEqual(['main','child','second']);
    rows.forEach((row,i)=>{row.getBoundingClientRect=()=>({top:i*50,bottom:i*50+50,height:50,left:0,right:300,width:300,x:0,y:i*50,toJSON(){}}) as DOMRect;});
    const grip=rows[0]!.querySelector<HTMLElement>('[data-linked-grip]')!;
    fireEvent.pointerDown(grip,{pointerId:1,button:0,pointerType:'mouse',clientX:5,clientY:25});
    fireEvent.pointerMove(document,{pointerId:1,clientX:5,clientY:140});
    expect(rows[0]).toHaveAttribute('data-dragging','true');
    expect(rows[2]).toHaveAttribute('data-drop','after');
    fireEvent.pointerUp(document,{pointerId:1,clientX:5,clientY:140});
    expect(actions.onMove).toHaveBeenCalledExactlyOnceWith(0,3);
    expect(rows[0]).not.toHaveAttribute('data-dragging');
    expect(screen.getByRole('status',{name:''}).textContent??'').toContain('linked-accounts-moved');
  });
  it('a press without movement does not reorder',()=>{
    const actions=mount();
    const grip=document.querySelector<HTMLElement>('[data-linked-grip]')!;
    fireEvent.pointerDown(grip,{pointerId:1,button:0,pointerType:'mouse',clientX:5,clientY:5});
    fireEvent.pointerUp(document,{pointerId:1,clientX:6,clientY:6});
    expect(actions.onMove).not.toHaveBeenCalled();
  });
  it('Alt+Arrow on the grip is the keyboard fallback, and announces the move',()=>{
    const actions=mount();
    const grips=screen.getAllByRole('button',{name:/^linked-accounts-move-handle/});
    fireEvent.keyDown(grips[0]!,{key:'ArrowUp',altKey:true});
    expect(actions.onMove).not.toHaveBeenCalled();
    fireEvent.keyDown(grips[0]!,{key:'ArrowDown',altKey:true});
    expect(actions.onMove).toHaveBeenCalledExactlyOnceWith(0,2);
    fireEvent.keyDown(grips[1]!,{key:'ArrowUp',altKey:true});
    expect(actions.onMove).toHaveBeenLastCalledWith(1,0);
    fireEvent.keyDown(grips[1]!,{key:'ArrowDown'});
    expect(actions.onMove).toHaveBeenCalledTimes(2);
  });
  it('edits a nickname inline: Enter saves, Esc restores, bad text is refused (t40 g)',async()=>{
    const actions=mount({...value,layout:{revision:'r1',order:['child','main'],nicknames:{child:'Work'}}});
    expect([...document.querySelectorAll<HTMLElement>('[data-linked-row]')].map(row=>row.dataset['key'])).toEqual(['child','main']);
    expect(screen.getByText('Work')).toBeVisible();
    const inputs=document.querySelectorAll<HTMLInputElement>('[data-linked-nickname]');
    expect(inputs[0]!.value).toBe('Work');
    fireEvent.input(inputs[0]!,{target:{value:'Office'}});
    fireEvent.keyDown(inputs[0]!,{key:'Escape'});
    expect(inputs[0]!.value).toBe('Work');
    expect(actions.onRename).not.toHaveBeenCalled();
    inputs[1]!.value='  Personal ';
    fireEvent.keyDown(inputs[1]!,{key:'Enter'});
    await Promise.resolve();
    expect(actions.onRename).toHaveBeenCalledExactlyOnceWith(null,'Personal');
    inputs[1]!.value='Bad\u202Ename';
    fireEvent.keyDown(inputs[1]!,{key:'Enter'});
    await Promise.resolve();
    expect(actions.onRename).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('alert')).toHaveTextContent('linked-accounts-nickname-invalid');
    inputs[0]!.value='';
    fireEvent.keyDown(inputs[0]!,{key:'Enter'});
    await Promise.resolve();
    expect(actions.onRename).toHaveBeenLastCalledWith('child','');
  });
});
