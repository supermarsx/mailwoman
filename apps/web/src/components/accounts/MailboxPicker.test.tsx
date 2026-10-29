import {cleanup,fireEvent,render,screen,within} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {MailboxPicker,MailboxPlain,type MailboxChoice} from './MailboxPicker.tsx';

vi.mock('../../i18n/index.ts',()=>({
  t:(key:string,args?:Record<string,string|number>)=>args?.address!==undefined?`${key}:${args.address}`:args?.count!==undefined?`${key}:${args.count}`:key,
  isolate:(value:string)=>value,
  loadCatalog:async()=>undefined,
}));
const settingsState=vi.hoisted(()=>({map:{} as Record<string,{value:unknown}>}));
vi.mock('../../api/settings.ts',()=>({settings:()=>settingsState.map}));
afterEach(()=>{cleanup();settingsState.map={};});

const choices:MailboxChoice[]=[
  {linkId:null,address:'me@example.org',nickname:'Personal',signIn:true,available:true,unread:4},
  {linkId:'work',address:'work@example.org',name:'Work',available:true,unread:null},
  {linkId:'old',address:'old@example.org',name:'Old',available:false,unread:2},
];
function mount(extra:Partial<Parameters<typeof MailboxPicker>[0]>={}){
  const onSelect=vi.fn(),onRenew=vi.fn();
  render(()=><MailboxPicker variant="full" choices={choices} activeLinkId={null} busy={false} onSelect={onSelect} onRenew={onRenew} {...extra}/>);
  const trigger=screen.getByRole('button',{name:'linked-accounts-select: Personal (me@example.org)'});
  return {onSelect,onRenew,trigger};
}
const tick=()=>Promise.resolve();

describe('mailbox picker (a switcher only, t48 k)',()=>{
  it('is a menu of radio items with avatar, name, address, unread count and active check, in the saved order',async()=>{
    const {trigger}=mount();
    expect(trigger).toHaveAttribute('aria-haspopup','menu');
    fireEvent.click(trigger);
    const menu=screen.getByRole('menu',{name:'linked-accounts-select'});
    expect(trigger).toHaveAttribute('aria-controls',menu.id);
    const items=within(menu).getAllByRole('menuitemradio');
    expect(items.map(item=>item.dataset['link'])).toEqual(['','work','old']);
    expect(items[0]).toHaveAttribute('aria-checked','true');
    expect(items[0]).toHaveTextContent('Personal');
    expect(items[0]).toHaveTextContent('me@example.org');
    expect(items[0]).toHaveTextContent('linked-accounts-unread:4');
    expect(items[0]!.querySelector('.person-avatar')).not.toBeNull();
    expect(items[2]).toHaveAttribute('aria-disabled','true');
    await tick();
    expect(document.activeElement).toBe(items[0]);
  });
  it('has no management at all: no grip, rename, primary star, renew or status text',()=>{
    const {trigger}=mount({choices:[choices[0]!,choices[1]!,{...choices[2]!,renew:true}]});
    fireEvent.click(trigger);
    const menu=screen.getByRole('menu');
    expect(within(menu).queryAllByRole('menuitem')).toHaveLength(0);
    expect(menu.querySelectorAll('[data-mailbox-grip],[data-mailbox-first],[data-mailbox-rename],[data-mailbox-renew],input')).toHaveLength(0);
    for(const item of within(menu).getAllByRole('menuitemradio'))expect(item.querySelectorAll('button')).toHaveLength(0);
    // The only status words are screen-reader text.
    for(const span of menu.querySelectorAll('span'))
      if(/^linked-accounts-(primary|primary-mailbox|first|unavailable|expired|renew.*)$/.test(span.textContent??''))expect(span.className).toMatch(/srOnly/);
    expect(menu.textContent).not.toContain('linked-accounts-first');
  });
  it('supports arrows, Home/End, type-ahead, Enter and Esc',async()=>{
    const {trigger,onSelect}=mount();
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
    fireEvent.keyDown(document.activeElement!,{key:'Home'});
    fireEvent.keyDown(document.activeElement!,{key:'w'});
    expect(document.activeElement).toBe(items[1]);
    // Alt+Arrow no longer reorders here (Settings does).
    fireEvent.keyDown(document.activeElement!,{key:'ArrowDown',altKey:true});
    expect(document.activeElement).toBe(items[2]);
    fireEvent.click(items[1]!);
    expect(onSelect).toHaveBeenCalledExactlyOnceWith('work');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(document.activeElement).toBe(trigger);
    fireEvent.click(trigger);
    await tick();
    fireEvent.keyDown(screen.getByRole('menu'),{key:'Escape'});
    expect(screen.queryByRole('menu')).toBeNull();
  });
  it('ignores the active and unavailable mailboxes and an outside press closes it',()=>{
    const {trigger,onSelect}=mount();
    fireEvent.click(trigger);
    const items=within(screen.getByRole('menu')).getAllByRole('menuitemradio');
    fireEvent.click(items[2]!);
    fireEvent.click(items[0]!);
    expect(onSelect).not.toHaveBeenCalled();
    fireEvent.click(trigger);
    fireEvent.pointerDown(document.body);
    expect(screen.queryByRole('menu')).toBeNull();
  });
  it('choosing a mailbox whose access expired opens renewal, never a dead end (t48 k)',async()=>{
    const {trigger,onSelect,onRenew}=mount({choices:[choices[0]!,choices[1]!,{...choices[2]!,renew:true}]});
    fireEvent.click(trigger);
    const old=within(screen.getByRole('menu')).getAllByRole('menuitemradio')[2]!;
    expect(old).not.toHaveAttribute('aria-disabled');
    expect(old.textContent).toContain('linked-accounts-expired');
    expect(old.querySelector('[class*="srOnly"]')?.textContent).toBe('linked-accounts-expired');
    fireEvent.click(old);
    expect(onRenew).toHaveBeenCalledExactlyOnceWith('old');
    expect(onSelect).not.toHaveBeenCalled();
    expect(screen.queryByRole('menu')).toBeNull();
  });
  it('the icon variant names the active linked mailbox and opens the same menu',()=>{
    render(()=><MailboxPicker variant="icon" choices={choices} activeLinkId="work" busy={false} onSelect={()=>undefined}/>);
    const button=screen.getByRole('button',{name:'mail-account-linked-active:work@example.org'});
    expect(button).toHaveTextContent('W');
    fireEvent.click(button);
    const menu=screen.getByRole('menu');
    expect(within(menu).getAllByRole('menuitemradio')[1]).toHaveAttribute('aria-checked','true');
    expect(within(menu).queryAllByRole('menuitem')).toHaveLength(0);
  });
  it('one mailbox is plain text, or a static letter, never a menu',()=>{
    render(()=><><MailboxPlain variant="full" address="me@example.org" unread={3}/><MailboxPlain variant="icon" address="me@example.org" unread={null}/></>);
    expect(screen.queryByRole('button')).toBeNull();
    expect(screen.getByText('me@example.org')).toBeVisible();
    expect(screen.getByRole('img',{name:'mail-account-active:me@example.org'})).not.toHaveAttribute('aria-haspopup');
  });
  // t39: the bubble beside the mailbox name is opt-in; the menu keeps its counts.
  it('shows no unread bubble beside the mailbox name by default, but keeps the menu counts',()=>{
    render(()=><><MailboxPlain variant="full" address="solo@example.org" unread={3}/><MailboxPicker variant="full" choices={choices} activeLinkId={null} busy={false} onSelect={vi.fn()}/></>);
    expect(document.querySelectorAll('[data-account-indicator] [data-mailbox-unread]')).toHaveLength(0);
    fireEvent.click(document.querySelector('button[data-account-indicator="picker"]')!);
    const menu=screen.getByRole('menu');
    expect(within(menu).getAllByRole('menuitemradio')[0]).toHaveTextContent('linked-accounts-unread:4');
  });
  it('shows the bubble beside the mailbox name when mail.accountUnreadBadge is on',()=>{
    settingsState.map={'mail.accountUnreadBadge':{value:true}};
    render(()=><><MailboxPlain variant="full" address="solo@example.org" unread={3}/><MailboxPicker variant="full" choices={choices} activeLinkId={null} busy={false} onSelect={vi.fn()}/></>);
    expect(document.querySelectorAll('[data-account-indicator] [data-mailbox-unread]')).toHaveLength(2);
  });
});
