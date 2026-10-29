import {describe,it,expect,vi} from 'vitest';
import {fireEvent,screen,within,waitFor} from '@solidjs/testing-library';
import {ConsolePath} from './ConsolePath.tsx';
import {renderInShell} from './shell/testkit.tsx';
import {validChosenPath,type ConsolePathApi,type ConsolePathView} from '../../api/admin/consolePath.ts';

const view=(over:Partial<ConsolePathView>={}):ConsolePathView=>({path:'/k7tq2xw9m4hrb3cz8npd',source:'generated',pending:null,pinnedBy:null,legacyDefault:false,minLength:12,...over});
const api=(data:ConsolePathView,save:ConsolePathApi['save']=vi.fn()):ConsolePathApi=>({read:async()=>({data,etag:'"one"',status:200}),save});

describe('console address (t76)',()=>{
  it('accepts only paths the server would accept',()=>{
    expect(validChosenPath('/ops-console-1',12)).toBe(true);
    expect(validChosenPath('/team/ops-console',12)).toBe(true);
    for(const bad of ['ops-console-12','/short','/ops console 1','/ops//console1','/a/b/c/d/eeeeeeee','/../abcdefghijkl'])expect(validChosenPath(bad,12),bad).toBe(false);
  });
  it('shows where the console is, says it hides rather than isolates, and confirms before saving',async()=>{
    const save=vi.fn(async()=>({data:view({pending:'/ops-console-t76x'}),etag:'"two"',status:200}));
    renderInShell(()=><ConsolePath readOnly={false} api={api(view(),save)}/>);
    expect(await screen.findByTestId('console-path-current')).toHaveTextContent('/k7tq2xw9m4hrb3cz8npd');
    expect(screen.getByText(/does not separate the console from the mail app/)).toBeVisible();
    const input=screen.getByLabelText('New path');
    fireEvent.input(input,{target:{value:'/short'}});
    expect(screen.getByRole('button',{name:'Save new address'})).toBeDisabled();
    fireEvent.input(input,{target:{value:'/ops-console-t76x'}});
    fireEvent.click(screen.getByRole('button',{name:'Save new address'}));
    expect(save).not.toHaveBeenCalled();
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:'Save new address'}));
    await waitFor(()=>expect(save).toHaveBeenCalledWith({path:'/ops-console-t76x'},'"one"'));
    expect(await screen.findByTestId('console-path-pending')).toHaveTextContent('/ops-console-t76x');
  });
  it('asks the server for a generated path',async()=>{
    const save=vi.fn(async()=>({data:view({pending:'/abcdefghijkmnpqrstuv'}),etag:'"two"',status:200}));
    renderInShell(()=><ConsolePath readOnly={false} api={api(view(),save)}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Generate a random path'}));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:'Save new address'}));
    await waitFor(()=>expect(save).toHaveBeenCalledWith({generate:true},'"one"'));
  });
  it('warns about /admin and offers no change while MW_ADMIN_PATH pins the path or the admin cannot write',async()=>{
    const first=renderInShell(()=><ConsolePath readOnly={false} api={api(view({path:'/admin',source:'env',pinnedBy:'MW_ADMIN_PATH',legacyDefault:true}))}/>);
    expect(await screen.findByText(/first address scanners try/)).toBeVisible();
    expect(screen.getByText(/Set by MW_ADMIN_PATH/)).toBeVisible();
    expect(screen.queryByLabelText('New path')).toBeNull();
    first.unmount();
    renderInShell(()=><ConsolePath readOnly api={api(view())}/>);
    await screen.findByTestId('console-path-current');
    expect(screen.queryByRole('button',{name:'Generate a random path'})).toBeNull();
  });
});
