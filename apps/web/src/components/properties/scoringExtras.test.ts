import {describe,expect,it,vi} from 'vitest';
import {trackerCountFor,trackerCountOfHtml,trainingState} from './scoringExtras.ts';

const marked=(n:number,tracker:number)=>Array.from({length:n},(_,i)=>`<img data-mw-blocked-host="h${i}.example"${i<tracker?' data-mw-tracker="1"':''}>`).join('');

describe('trackers',()=>{
 it('counts the tracker markers the sanitiser leaves',async()=>{
  const sanitize=vi.fn(async(request:{html:string})=>request.html) as never;
  expect(await trackerCountOfHtml(marked(4,3),sanitize)).toBe(3);
  expect(await trackerCountOfHtml('',sanitize)).toBe(0);
  expect(await trackerCountOfHtml('<p>plain</p>',sanitize)).toBe(0);
 });
 it('fetches the body and returns null when it cannot',async()=>{
  const sanitize=vi.fn(async(request:{html:string})=>request.html) as never;
  const ok={jmap:vi.fn(async()=>({methodResponses:[['Email/get',{list:[{id:'m',htmlBody:[{partId:'1',type:'text/html'}],bodyValues:{'1':{value:marked(2,2)}}}]},'g']]}))};
  expect(await trackerCountFor(ok as never,'a','m',sanitize)).toBe(2);
  expect(await trackerCountFor({jmap:vi.fn(async()=>{throw new Error('x');})} as never,'a','m',sanitize)).toBeNull();
 });
});

describe('spam filter training',()=>{
 it('reads the keywords first, then the filter autolearn note',()=>{
  expect(trainingState({keywords:{$junk:true}},null)).toBe('spam');
  expect(trainingState({keywords:{$notjunk:true}},null)).toBe('ham');
  expect(trainingState({keywords:{}},'No, score=0.1 required=5.0 tests=NONE autolearn=ham autolearn_force=no')).toBe('ham');
  expect(trainingState({keywords:{}},'Yes, score=9 autolearn=spam')).toBe('spam');
  expect(trainingState({keywords:{}},'No, score=0.1 autolearn=no')).toBe('untrained');
  expect(trainingState({},null)).toBe('untrained');
 });
});
