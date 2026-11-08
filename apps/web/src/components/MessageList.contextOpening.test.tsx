import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {mkEmail,renderWithApp} from './appHarness.tsx';

afterEach(()=>{cleanup();localStorage.clear();});

const row=()=>screen.getByText('Context target',{selector:'.list__subject'}).closest('button')!;

it('routes context Open and Open in a new tab separately without changing another selection',async()=>{
  const open=vi.fn(),tab=vi.fn();
  const {app}=renderWithApp(()=><MessageList onOpenMessage={open} onOpenMessageTab={tab}/>,{emails:[mkEmail('a'),mkEmail('b',{subject:'Context target'})]});
  await app.login({username:'me@example.org',password:'p'});
  await waitFor(()=>expect(app.messages()).toHaveLength(2));
  app.setSelection(['a']);
  const directOpen=vi.spyOn(app,'openMessage');
  fireEvent.contextMenu(row());
  fireEvent.click(screen.getByRole('menuitem',{name:'Open message'}));
  expect(open.mock.calls).toEqual([['b']]);
  expect(tab).not.toHaveBeenCalled();
  expect(directOpen).not.toHaveBeenCalled();
  fireEvent.contextMenu(row());
  fireEvent.click(screen.getByRole('menuitem',{name:'Open in a new tab'}));
  expect(tab.mock.calls).toEqual([['b']]);
  expect(open).toHaveBeenCalledTimes(1);
  expect(app.selectedIds()).toEqual(new Set(['a']));
});

it('omits the new-tab action when the host cannot open message tabs',async()=>{
  const {app}=renderWithApp(()=><MessageList/>,{emails:[mkEmail('b',{subject:'Context target'})]});
  await app.login({username:'me@example.org',password:'p'});
  await waitFor(()=>expect(app.messages()).toHaveLength(1));
  const directOpen=vi.spyOn(app,'openMessage').mockResolvedValue();
  fireEvent.contextMenu(row());
  expect(screen.queryByRole('menuitem',{name:'Open in a new tab'})).toBeNull();
  fireEvent.click(screen.getByRole('menuitem',{name:'Open message'}));
  expect(directOpen).toHaveBeenCalledWith('b');
});
