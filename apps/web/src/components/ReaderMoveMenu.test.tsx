import { describe, expect, it, vi } from 'vitest';
import { moveActions } from './ReaderMoveMenu.tsx';
import type { Email } from '../api/jmap-types.ts';

describe('reader Move to folder', () => {
  const app = {
    mailboxes: () => [
      { id: 'inbox', name: 'Inbox', role: 'inbox' },
      { id: 'work', name: 'Work' },
      { id: 'saved', name: 'Saved search', mailwomanSearchQuery: 'x' },
    ],
    moveMessage: vi.fn(),
  } as never;

  it('offers every real folder except the ones the message is already in and saved searches', () => {
    const actions = moveActions({ id: 'm1', mailboxIds: { inbox: true } } as unknown as Email, app);
    expect(actions.map((a) => a.label)).toEqual(['Work']);
  });

  it('moves the open message to the chosen folder', () => {
    const [work] = moveActions({ id: 'm1', mailboxIds: { inbox: true } } as unknown as Email, app);
    work!.run?.();
    expect((app as { moveMessage: ReturnType<typeof vi.fn> }).moveMessage).toHaveBeenCalledWith('m1', 'work');
  });
});

it('moves every selected message, including when one already belongs to the destination',()=>{
 const first={id:'one',mailboxIds:{inbox:true}} as unknown as Email;
 const second={id:'two',mailboxIds:{work:true}} as unknown as Email;
 const moveMessage=vi.fn(),selected=new Set(['one','two']);
 const app={mailboxes:()=>[{id:'inbox',name:'Inbox'},{id:'work',name:'Work'},{id:'denied',name:'Denied',myRights:{mayAddItems:false}}],messages:()=>[first,second],selectedIds:()=>selected,moveMessage} as never;
 const destination=moveActions(first,app).find(action=>action.id==='move:work');expect(destination).toBeDefined();destination!.run();
 expect(moveMessage).toHaveBeenCalledWith(['one','two'],'work');expect(moveActions(first,app).some(action=>action.id==='move:denied')).toBe(false);
});
it('keeps an unselected context message separate from an existing selection',()=>{
 const email={id:'other',mailboxIds:{inbox:true}} as unknown as Email,moveMessage=vi.fn();
 const app={mailboxes:()=>[{id:'work',name:'Work'}],messages:()=>[],selectedIds:()=>new Set(['one','two']),moveMessage} as never;
 moveActions(email,app)[0]!.run();expect(moveMessage).toHaveBeenCalledWith('other','work');
});
