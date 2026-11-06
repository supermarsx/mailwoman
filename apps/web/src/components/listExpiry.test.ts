import {describe,expect,it} from 'vitest';
import {rowExpired,visibleRows} from './listExpiry.ts';

const now=new Date('2026-11-06T12:00:00Z');
const past={'header:Expires:asText':'Thu, 05 Nov 2026 09:00:00 +0000'} as never;
const future={'header:Expires:asText':'Fri, 06 Nov 2026 18:00:00 +0000'} as never;
const none={} as never;

describe('expired messages in the list',()=>{
 it('marks only past dates, and only when the mode is not off',()=>{
  expect(rowExpired(past,now,'dim')).toBe(true);
  expect(rowExpired(future,now,'dim')).toBe(false);
  expect(rowExpired(none,now,'dim')).toBe(false);
  expect(rowExpired(past,now,'off')).toBe(false);
 });
 it('drops expired rows only in hide mode',()=>{
  const rows=[past,future,none];
  expect(visibleRows(rows,now,'hide')).toEqual([future,none]);
  expect(visibleRows(rows,now,'dim')).toEqual(rows);
  expect(visibleRows(rows,now,'archive-suggestion')).toEqual(rows);
 });
});

