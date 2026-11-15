import {expect,it} from 'vitest';
import {matchesCachedSearch} from './querySearch.ts';
import type {Email} from '../api/jmap-types.ts';
const email={id:'a',subject:'Quarterly invoice',preview:'Preview',receivedAt:'2026-11-15T00:00:00Z',size:4096,from:[{email:'alice@example.test'}],to:[],keywords:{$flagged:true},mailboxIds:{inbox:true},bodyValues:{body:{value:'<p>Payment received</p>'}}} as unknown as Email;
it('supports boolean groups, negation, addresses, body text and follow-up filters on cached data',()=>{
 expect(matchesCachedSearch(email,'(from:alice OR from:bob) AND body:"Payment received"')).toBe(true);
 expect(matchesCachedSearch(email,'subject:invoice -from:bob is:followup larger:1k before:2026-12-01')).toBe(true);
 expect(matchesCachedSearch(email,'NOT (from:alice OR from:bob)')).toBe(false);
 expect(matchesCachedSearch(email,'is:finished')).toBe(false);
 expect(matchesCachedSearch(email,'subject:"Invoice missing" OR body:received')).toBe(true);
 expect(matchesCachedSearch(email,'subject:invoice AND (')).toBe(false);
});
