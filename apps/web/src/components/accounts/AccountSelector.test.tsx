import {fireEvent,render,screen,cleanup} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {AccountSelector} from './AccountSelector.tsx';
vi.mock('../../i18n/index.ts',()=>({t:(key:string)=>key}));
afterEach(cleanup);
const choices=[{linkId:null,address:'primary@example.org',available:true},{linkId:'child',address:'child@example.org',available:true},{linkId:'expired',address:'expired@example.org',available:false}];
describe('mailbox account selector',()=>{
  it('requests a server link ID but retains the confirmed selection while switching',()=>{
    const select=vi.fn();
    render(()=><AccountSelector choices={choices} activeLinkId={null} busy={false} onSelect={select} onManage={()=>undefined}/>);
    const input=screen.getByRole('combobox') as HTMLSelectElement;
    fireEvent.change(input,{target:{value:'child'}});
    expect(select).toHaveBeenCalledExactlyOnceWith('child');
    expect(input.value).toBe('');
  });
  it('refuses unavailable and unknown link IDs, with a separate management action',()=>{
    const select=vi.fn(),manage=vi.fn();
    render(()=><AccountSelector choices={choices} activeLinkId={null} busy={false} onSelect={select} onManage={manage}/>);
    fireEvent.change(screen.getByRole('combobox'),{target:{value:'expired'}});
    expect(select).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-manage'}));
    expect(manage).toHaveBeenCalledOnce();
  });
  it('disables selection during a pending verification and presents failure text',()=>{
    render(()=><AccountSelector choices={choices} activeLinkId="child" busy={true} error="Verification failed" onSelect={()=>undefined} onManage={()=>undefined}/>);
    expect(screen.getByRole('combobox')).toBeDisabled();
    expect(screen.getByRole('button')).toBeDisabled();
    expect(screen.getByRole('alert')).toHaveTextContent('Verification failed');
  });
});
