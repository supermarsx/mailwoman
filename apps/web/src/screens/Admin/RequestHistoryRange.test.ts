import {describe,it,expect} from 'vitest';
import {customHistoryRange,historyLocalInput,relativeHistoryRange} from './RequestHistoryRange.ts';

describe('request history ranges',()=>{
 it('refreshes relative windows at the time of the request',()=>{
  expect(relativeHistoryRange(21600,86400123)).toEqual({from:64800,to:86400});
  expect(relativeHistoryRange(21600,90000123)).toEqual({from:68400,to:90000});
 });
 it('round-trips local date inputs and rejects missing or normalized dates',()=>{
  const end=new Date(2026,9,26,14,30).getTime();
  const to=historyLocalInput(end/1000),from=historyLocalInput(end/1000-3600);
  expect(customHistoryRange(from,to,3600,end)).toEqual({range:{from:end/1000-3600,to:end/1000}});
  expect(customHistoryRange('',to,3600,end).error).toBe('invalid');
  expect(customHistoryRange('2026-02-30T12:00',to,3600,end).error).toBe('invalid');
 });
 it('refuses reversed, future and over-limit custom periods',()=>{
  const now=new Date(2026,9,26,14,30).getTime(),end=historyLocalInput(now/1000),start=historyLocalInput(now/1000-7200);
  expect(customHistoryRange(end,start,86400,now).error).toBe('order');
  expect(customHistoryRange(start,end,3600,now).error).toBe('limit');
  expect(customHistoryRange(start,historyLocalInput(now/1000+60),86400,now).error).toBe('future');
 });
});
