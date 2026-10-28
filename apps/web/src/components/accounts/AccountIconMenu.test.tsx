import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {AccountIconMenu} from './AccountIconMenu.tsx';
import {CollapsibleBar} from '../bars/index.ts';
import type {BarModeHandle} from '../bars/index.ts';

vi.mock('../../i18n/index.ts',()=>({
  t:(key:string,args?:Record<string,string>)=>args?.address?`${key}:${args.address}`:key,
  isolate:(value:string)=>value,
  loadCatalog:async()=>undefined,
}));
afterEach(cleanup);

function autoBar():BarModeHandle{
  return {mode:()=>'auto',locked:()=>false,saving:()=>false,error:()=>null,setMode:async()=>true} as unknown as BarModeHandle;
}

describe('account icon menu',()=>{
  it('names the active address and marks a linked mailbox',()=>{
    render(()=><AccountIconMenu address="child@example.org" linked>{()=><span>choices</span>}</AccountIconMenu>);
    const button=screen.getByRole('button',{name:'mail-account-linked-active:child@example.org'});
    expect(button).toHaveTextContent('C');
    expect(button).toHaveAttribute('data-linked','true');
    expect(button).toHaveAttribute('aria-expanded','false');
  });
  it('opens the same choices in a popover, and Esc closes it and restores focus',async()=>{
    render(()=><AccountIconMenu address="me@example.org" linked={false}>{close=><button type="button" onClick={close}>Manage</button>}</AccountIconMenu>);
    const button=screen.getByRole('button',{name:'mail-account-active:me@example.org'});
    fireEvent.click(button);
    const dialog=screen.getByRole('dialog',{name:'mail-account-active:me@example.org'});
    expect(button).toHaveAttribute('aria-expanded','true');
    expect(button).toHaveAttribute('aria-controls',dialog.id);
    await Promise.resolve();
    expect(document.activeElement).toBe(screen.getByRole('button',{name:'Manage'}));
    fireEvent.keyDown(dialog,{key:'Escape'});
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(document.activeElement).toBe(button);
  });
  it('closes on an outside press and through the close callback',()=>{
    render(()=><AccountIconMenu address="me@example.org" linked={false}>{close=><button type="button" onClick={close}>Manage</button>}</AccountIconMenu>);
    const button=screen.getByRole('button',{name:'mail-account-active:me@example.org'});
    fireEvent.click(button);
    fireEvent.pointerDown(document.body);
    expect(screen.queryByRole('dialog')).toBeNull();
    fireEvent.click(button);
    fireEvent.click(screen.getByRole('button',{name:'Manage'}));
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  it('holds an auto-hide bar open while the popover is open',()=>{
    vi.useFakeTimers();
    try{
      const {container}=render(()=><CollapsibleBar id="mailTopBar" revealPolicy="overlay" bar={autoBar()}>
        <div role="toolbar" aria-label="Workspace layout"><AccountIconMenu address="me@example.org" linked={false}>{()=><span>choices</span>}</AccountIconMenu></div>
      </CollapsibleBar>);
      const root=container.querySelector('[data-bar="mailTopBar"]')!;
      fireEvent.click(container.querySelector('[data-bar-handle]')!);
      expect(root).toHaveAttribute('data-expanded','true');
      fireEvent.click(screen.getByRole('button',{name:'mail-account-active:me@example.org'}));
      // The popover lives outside the bar (a portal): pressing in it must not close the bar.
      fireEvent.pointerDown(screen.getByRole('dialog'));
      vi.advanceTimersByTime(1000);
      expect(root).toHaveAttribute('data-expanded','true');
    }finally{vi.useRealTimers();}
  });
});
