import {cleanup,fireEvent,render,screen,within} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {MailboxPicker,MailboxPlain,type MailboxChoice} from './MailboxPicker.tsx';
import {installPointerEvent} from './pointerEventPolyfill.test-util.ts';
installPointerEvent();

vi.mock('../../i18n/index.ts',()=>({
  t:(key:string,args?:Record<string,string|number>)=>args?.address!==undefined?`${key}:${args.address}`:args?.count!==undefined?`${key}:${args.count}`:args?.name!==undefined?`${key}:${args.name}@${args.position}`:key,
  isolate:(value:string)=>value,
  loadCatalog:async()=>undefined,
}));
afterEach(cleanup);

const choices:MailboxChoice[]=[
  {linkId:null,address:'me@example.org',nickname:'Personal',signIn:true,available:true,unread:4},
  {linkId:'work',address:'work@example.org',name:'Work',available:true,unread:null},
  {linkId:'old',address:'old@example.org',name:'Old',available:false,unread:2},
];
function mount(extra:Partial<Parameters<typeof MailboxPicker>[0]>={}){
  const onSelect=vi.fn(),onSetFirst=vi.fn(),onMove=vi.fn(),onRename=vi.fn(async()=>true);
  render(()=><MailboxPicker variant="full" choices={choices} activeLinkId={null} firstLinkId={null} busy={false} onSelect={onSelect} onSetFirst={onSetFirst} onMove={onMove} onRename={onRename} {...extra}/>);
  const trigger=screen.getByRole('button',{name:'linked-accounts-select: Personal (me@example.org)'});
  return {onSelect,onSetFirst,onMove,onRename,trigger};
}
const tick=()=>Promise.resolve();

describe('mailbox picker',()=>{
  it('is a menu of radio items with avatar, name, address, unread count, active check and primary marker',async()=>{
    const {trigger}=mount();
    expect(trigger).toHaveAttribute('aria-haspopup','menu');
    expect(trigger).toHaveAttribute('aria-expanded','false');
    fireEvent.click(trigger);
    const menu=screen.getByRole('menu',{name:'linked-accounts-select'});
    expect(trigger).toHaveAttribute('aria-controls',menu.id);
    const items=within(menu).getAllByRole('menuitemradio');
    expect(items).toHaveLength(3);
    expect(items[0]).toHaveAttribute('aria-checked','true');
    expect(items[1]).toHaveAttribute('aria-checked','false');
    expect(items[0]).toHaveTextContent('Personal');
    expect(items[0]).toHaveTextContent('linked-accounts-primary');
    expect(items[0]).toHaveTextContent('me@example.org');
    expect(items[0]).toHaveTextContent('linked-accounts-unread:4');
    expect(items[0]!.querySelector('.person-avatar')).not.toBeNull();
    expect(items[0]!.querySelector('[data-mailbox-primary]')).not.toBeNull();
    expect(items[1]!.querySelector('[data-mailbox-primary]')).toBeNull();
    expect(items[2]).toHaveAttribute('aria-disabled','true');
    // Set-as-primary only on Work (the sign-in account is primary, Old is
    // unavailable); Rename on all three.
    expect(within(menu).getAllByRole('menuitem',{name:/set-first/})).toHaveLength(1);
    expect(within(menu).getAllByRole('menuitem',{name:/linked-accounts-rename/})).toHaveLength(3);
    await tick();
    expect(document.activeElement).toBe(items[0]);
  });
  it('supports arrows, Home/End, Right/Left, type-ahead, Enter and Esc',async()=>{
    const {trigger,onSelect,onSetFirst}=mount();
    trigger.focus();
    fireEvent.keyDown(trigger,{key:'ArrowDown'});
    await tick();
    const menu=screen.getByRole('menu');
    const items=within(menu).getAllByRole('menuitemradio');
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(document.activeElement!,{key:'ArrowDown'});
    expect(document.activeElement).toBe(items[1]);
    fireEvent.keyDown(document.activeElement!,{key:'End'});
    expect(document.activeElement).toBe(items[2]);
    fireEvent.keyDown(document.activeElement!,{key:'ArrowDown'});
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(document.activeElement!,{key:'ArrowUp'});
    expect(document.activeElement).toBe(items[2]);
    fireEvent.keyDown(document.activeElement!,{key:'Home'});
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(document.activeElement!,{key:'w'});
    expect(document.activeElement).toBe(items[1]);
    fireEvent.keyDown(document.activeElement!,{key:'ArrowRight'});
    const action=within(menu).getByRole('menuitem',{name:'linked-accounts-set-first:work@example.org'});
    expect(document.activeElement).toBe(action);
    fireEvent.click(action);
    expect(onSetFirst).toHaveBeenCalledExactlyOnceWith('work');
    fireEvent.keyDown(action,{key:'ArrowLeft'});
    expect(document.activeElement).toBe(items[1]);
    fireEvent.click(items[1]!);
    expect(onSelect).toHaveBeenCalledExactlyOnceWith('work');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(document.activeElement).toBe(trigger);
    fireEvent.click(trigger);
    await tick();
    fireEvent.keyDown(screen.getByRole('menu'),{key:'Escape'});
    expect(screen.queryByRole('menu')).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });
  it('ignores the active and unavailable mailboxes and an outside press closes it',()=>{
    const {trigger,onSelect}=mount();
    fireEvent.click(trigger);
    const items=within(screen.getByRole('menu')).getAllByRole('menuitemradio');
    fireEvent.click(items[2]!);
    expect(onSelect).not.toHaveBeenCalled();
    fireEvent.click(items[0]!);
    expect(onSelect).not.toHaveBeenCalled();
    fireEvent.click(trigger);
    fireEvent.pointerDown(document.body);
    expect(screen.queryByRole('menu')).toBeNull();
  });
  it('the icon variant names the active linked mailbox and opens the same menu',()=>{
    render(()=><MailboxPicker variant="icon" choices={choices} activeLinkId="work" firstLinkId="work" busy={false} onSelect={()=>undefined}/>);
    const button=screen.getByRole('button',{name:'mail-account-linked-active:work@example.org'});
    expect(button).toHaveTextContent('W');
    expect(button).toHaveAttribute('data-linked','true');
    fireEvent.click(button);
    const menu=screen.getByRole('menu');
    expect(within(menu).getAllByRole('menuitemradio')[1]).toHaveAttribute('aria-checked','true');
    // No onSetFirst: no primary actions.
    expect(within(menu).queryAllByRole('menuitem')).toHaveLength(0);
  });
  it('one mailbox is plain text, or a static letter, never a menu',()=>{
    render(()=><><MailboxPlain variant="full" address="me@example.org" unread={3}/><MailboxPlain variant="icon" address="me@example.org" unread={null}/></>);
    expect(screen.queryByRole('button')).toBeNull();
    expect(screen.getByText('me@example.org')).toBeVisible();
    expect(screen.getByRole('img',{name:'mail-account-active:me@example.org'})).not.toHaveAttribute('aria-haspopup');
  });
  it('a drag in the menu reorders and never switches; Alt+Arrow moves too',async()=>{
    const {trigger,onSelect,onMove}=mount();
    fireEvent.click(trigger);
    await tick();
    const menu=screen.getByRole('menu');
    const rows=[...menu.querySelectorAll<HTMLElement>('[data-mailbox-row]')];
    rows.forEach((row,i)=>{row.getBoundingClientRect=()=>({top:i*44,bottom:i*44+44,height:44,left:0,right:300,width:300,x:0,y:i*44,toJSON(){}}) as DOMRect;});
    const work=within(menu).getAllByRole('menuitemradio')[1]!;
    fireEvent.pointerDown(work,{pointerId:3,button:0,pointerType:'mouse',clientX:50,clientY:66});
    fireEvent.pointerMove(document,{pointerId:3,clientX:50,clientY:10});
    expect(rows[1]).toHaveAttribute('data-dragging','true');
    expect(rows[0]).toHaveAttribute('data-drop','before');
    fireEvent.pointerUp(document,{pointerId:3,clientX:50,clientY:10});
    fireEvent.click(work);
    expect(onMove).toHaveBeenCalledExactlyOnceWith(1,0);
    expect(onSelect).not.toHaveBeenCalled();
    expect(screen.getByRole('menu')).toBeInTheDocument();
    // Touch: a long press is needed; moving first is a scroll, not a drag.
    fireEvent.pointerDown(work,{pointerId:4,button:0,pointerType:'touch',clientX:50,clientY:66});
    fireEvent.pointerMove(document,{pointerId:4,clientX:50,clientY:20});
    fireEvent.pointerUp(document,{pointerId:4,clientX:50,clientY:20});
    expect(onMove).toHaveBeenCalledTimes(1);
    work.focus();
    fireEvent.keyDown(work,{key:'ArrowDown',altKey:true});
    expect(onMove).toHaveBeenLastCalledWith(1,3);
    expect(screen.getByRole('status').textContent).toContain('linked-accounts-moved:Work@3');
  });
  it('a long press then drag reorders on touch',()=>{
    vi.useFakeTimers();
    try{
      const {trigger,onMove,onSelect}=mount();
      fireEvent.click(trigger);
      const menu=screen.getByRole('menu');
      const rows=[...menu.querySelectorAll<HTMLElement>('[data-mailbox-row]')];
      rows.forEach((row,i)=>{row.getBoundingClientRect=()=>({top:i*44,bottom:i*44+44,height:44,left:0,right:300,width:300,x:0,y:i*44,toJSON(){}}) as DOMRect;});
      const me=within(menu).getAllByRole('menuitemradio')[0]!;
      fireEvent.pointerDown(me,{pointerId:5,button:0,pointerType:'touch',clientX:50,clientY:20});
      vi.advanceTimersByTime(400);
      expect(rows[0]).toHaveAttribute('data-dragging','true');
      fireEvent.pointerMove(document,{pointerId:5,clientX:50,clientY:120});
      fireEvent.pointerUp(document,{pointerId:5,clientX:50,clientY:120});
      fireEvent.click(me);
      expect(onMove).toHaveBeenCalledExactlyOnceWith(0,3);
      expect(onSelect).not.toHaveBeenCalled();
    }finally{vi.useRealTimers();}
  });
  it('renames inline: Esc cancels, Enter saves, refused text stays open',async()=>{
    const {trigger,onRename}=mount();
    fireEvent.click(trigger);
    await tick();
    const menu=screen.getByRole('menu');
    fireEvent.click(within(menu).getByRole('menuitem',{name:'linked-accounts-rename:work@example.org'}));
    let input=within(menu).getByRole('textbox',{name:'linked-accounts-nickname-for:work@example.org'}) as HTMLInputElement;
    fireEvent.input(input,{target:{value:'Office'}});
    fireEvent.keyDown(input,{key:'Escape'});
    expect(within(menu).queryByRole('textbox')).toBeNull();
    expect(screen.getByRole('menu')).toBeInTheDocument();
    expect(onRename).not.toHaveBeenCalled();
    fireEvent.click(within(menu).getByRole('menuitem',{name:'linked-accounts-rename:work@example.org'}));
    input=within(menu).getByRole('textbox') as HTMLInputElement;
    input.value='x'.repeat(65);
    fireEvent.keyDown(input,{key:'Enter'});
    await tick();
    expect(onRename).not.toHaveBeenCalled();
    expect(within(menu).getByRole('alert')).toHaveTextContent('linked-accounts-nickname-invalid');
    input.value='  Office ';
    fireEvent.keyDown(input,{key:'Enter'});
    await tick();await tick();
    expect(onRename).toHaveBeenCalledExactlyOnceWith('work','Office');
    expect(within(menu).queryByRole('textbox')).toBeNull();
  });
});
