import {expect,it} from 'vitest';
import {moveNeighbour,type MovedMessages} from './moveEvents.ts';
const event:MovedMessages={accountId:'a',mailboxId:'inbox',sort:'newest',ids:['b','c'],order:['a','b','c','d'],mode:'next'};
it('skips bulk-moved rows, follows display order and falls back at either end',()=>{
 const available=new Set(['a','d']);expect(moveNeighbour('b',event,available)).toBe('d');
 expect(moveNeighbour('c',{...event,mode:'previous'},available)).toBe('a');
 expect(moveNeighbour('d',{...event,ids:['d']},new Set(['a']))).toBe('a');
 expect(moveNeighbour('a',{...event,ids:['a'],mode:'previous'},new Set(['d']))).toBe('d');
 expect(moveNeighbour('b',event,new Set())).toBeUndefined();expect(moveNeighbour('missing',event,available)).toBeUndefined();
 for(const mode of ['keep','close'] as const)expect(moveNeighbour('b',{...event,mode},available)).toBeUndefined();
});
