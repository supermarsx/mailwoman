import {describe,it,expect} from 'vitest';
import {validTimeZone,wallTimeCandidates} from './timezones.ts';
describe('explicit time zones',()=>{
  it('validates zones without treating an address or offset as a location',()=>{
    expect(validTimeZone('Europe/Lisbon')).toBe(true);expect(validTimeZone('UTC')).toBe(true);
    for(const value of ['','someone@example.org','+01:00','Mars/Olympus'])expect(validTimeZone(value)).toBe(false);
  });
  it('resolves ordinary and half-hour offsets',()=>{
    expect(wallTimeCandidates('2026-07-13T09:00','Europe/Lisbon').map(date=>date.toISOString())).toEqual(['2026-07-13T08:00:00.000Z']);
    expect(wallTimeCandidates('2026-07-13T09:00','Asia/Kolkata').map(date=>date.toISOString())).toEqual(['2026-07-13T03:30:00.000Z']);
  });
  it('does not silently normalize nonexistent or repeated DST times',()=>{
    expect(wallTimeCandidates('2026-03-29T01:30','Europe/London')).toEqual([]);
    expect(wallTimeCandidates('2026-10-25T01:30','Europe/London').map(date=>date.toISOString())).toEqual(['2026-10-25T00:30:00.000Z','2026-10-25T01:30:00.000Z']);
    expect(wallTimeCandidates('2026-02-30T12:00','UTC')).toEqual([]);
    expect(wallTimeCandidates('2011-12-30T12:00','Pacific/Apia')).toEqual([]);
  });
});
