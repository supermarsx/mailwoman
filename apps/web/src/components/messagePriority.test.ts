import {afterEach,describe,expect,it} from 'vitest';
import {backfillPriority,forgetBackfilledPriorities,priorityFromHeaders,priorityOf} from './messagePriority.ts';

afterEach(()=>forgetBackfilledPriorities());

describe('priority from raw headers',()=>{
 it('reads X-Priority numbers and words',()=>{
  expect(priorityFromHeaders('X-Priority: 1 (Highest)\r\n\r\nbody')).toBe('high');
  expect(priorityFromHeaders('X-Priority: 2\r\n\r\n')).toBe('high');
  expect(priorityFromHeaders('X-Priority: 5 (Lowest)\r\n\r\n')).toBe('low');
  expect(priorityFromHeaders('X-Priority: 3 (Normal)\r\n\r\n')).toBe('normal');
 });
 it('falls through Importance, X-MSMail-Priority and Priority',()=>{
  expect(priorityFromHeaders('Importance: High\r\n\r\n')).toBe('high');
  expect(priorityFromHeaders('X-MSMail-Priority: Low\r\n\r\n')).toBe('low');
  expect(priorityFromHeaders('Priority: urgent\r\n\r\n')).toBe('high');
  expect(priorityFromHeaders('Priority: non-urgent\r\n\r\n')).toBe('low');
  expect(priorityFromHeaders('Subject: x\r\n\r\n')).toBe('normal');
 });
 it('uses the first header that says something, and ignores the body',()=>{
  expect(priorityFromHeaders('X-Priority: 3\r\nImportance: High\r\n\r\n')).toBe('normal');
  expect(priorityFromHeaders('Subject: x\r\n\r\nImportance: High\r\n')).toBe('normal');
  expect(priorityFromHeaders('Importance:\r\n high\r\n\r\n')).toBe('high');
 });
});

describe('lazy backfill for messages stored before the parser',()=>{
 it('shows no mark until the message is opened, then marks it',()=>{
  expect(priorityOf({id:'m1'})).toBe('normal');
  backfillPriority('m1','Importance: High\r\n\r\n');
  expect(priorityOf({id:'m1'})).toBe('high');
  expect(priorityOf({id:'m2'})).toBe('normal');
 });
 it('never overrides what the server already read',()=>{
  backfillPriority('m1','Importance: High\r\n\r\n');
  expect(priorityOf({id:'m1',mailwomanPriority:'low'})).toBe('low');
 });
 it('keeps the answer for the session',()=>{
  backfillPriority('m3','X-Priority: 5\r\n\r\n');
  expect(JSON.parse(sessionStorage.getItem('mw.priority.backfill')??'{}')).toEqual({m3:'low'});
 });
});
