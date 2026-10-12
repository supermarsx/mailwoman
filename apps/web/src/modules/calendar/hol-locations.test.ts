// `hol-locations.ts`: the sections of a holiday pack, and a pack cut down to a
// choice of them.
//
// The header cases are the engine's `is_section_header`
// (crates/mw-ics/src/hol.rs): trimmed, opens with `[`, a `]` after it, a
// non-empty name between. `crates/mw-engine/tests/t30_calendar_import.rs` holds
// the engine to the same inputs where it matters to the import: a section with
// no name is no section, and lines before the first section are not read.

import { describe, it, expect } from 'vitest';
import { holLocations, holPack } from './hol-locations.ts';

// Lines in the layout of Outlook's own file, including one with a calendar-type
// id; that line is carried along untouched (the engine decides it is no date).
const THREE = [
  '[Argentina] 3',
  'San Martín National Patriot Day,2026/8/17',
  'Eid al Adha,1447/12/10,6',
  "New Year's Day,2026/1/1",
  '[Portugal] 1',
  'Dia de Portugal,2026/6/10',
  '[Japan] 1',
  'Shōwa Day,2026/4/29',
  '',
].join('\r\n');

describe('holLocations', () => {
  it('lists the sections in file order, each with its own lines', () => {
    const locations = holLocations(THREE);
    expect(locations.map((l) => l.name)).toEqual(['Argentina', 'Portugal', 'Japan']);
    expect(locations[0]!.lines).toEqual([
      '[Argentina] 3',
      'San Martín National Patriot Day,2026/8/17',
      'Eid al Adha,1447/12/10,6',
      "New Year's Day,2026/1/1",
    ]);
    expect(locations[1]!.lines).toEqual(['[Portugal] 1', 'Dia de Portugal,2026/6/10']);
  });

  it('reads LF line ends as well as CRLF', () => {
    expect(holLocations(THREE.replace(/\r\n/g, '\n')).map((l) => l.name)).toEqual(['Argentina', 'Portugal', 'Japan']);
  });

  it('a pack with one section has one location', () => {
    expect(holLocations('[Test Holidays] 3\nNew Year,2026/01/01\n').map((l) => l.name)).toEqual(['Test Holidays']);
  });

  it('takes a header by the same rule as the engine', () => {
    const names = (text: string): string[] => holLocations(text).map((l) => l.name);
    expect(names('  [ Spaced ] 2  \r\nA,2026/1/1\r\n')).toEqual(['Spaced']);
    expect(names('[No count]\r\nA,2026/1/1\r\n')).toEqual(['No count']);
    // No name, no closing bracket, or the bracket not first: not a header.
    expect(names('[] 1\r\nA,2026/1/1\r\n')).toEqual([]);
    expect(names('[  ] 1\r\nA,2026/1/1\r\n')).toEqual([]);
    expect(names('[Open 1\r\nA,2026/1/1\r\n')).toEqual([]);
    expect(names('note [Portugal] 1\r\nA,2026/1/1\r\n')).toEqual([]);
  });

  it('gives lines before the first section to no location', () => {
    const locations = holLocations('Early,2026/1/1\r\n[Holidays] 1\r\nLate,2026/2/2\r\n');
    expect(locations).toHaveLength(1);
    expect(locations[0]!.lines).toEqual(['[Holidays] 1', 'Late,2026/2/2', '']);
  });

  it('finds none in text that is not a pack, and none in an iCalendar document', () => {
    expect(holLocations('not a calendar')).toEqual([]);
    expect(holLocations('')).toEqual([]);
    // A bracketed line inside an iCalendar document is not a location.
    expect(holLocations('BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:x\r\n[a] 1\r\n[b] 1\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n')).toEqual([]);
  });
});

describe('holPack', () => {
  it('holds the chosen sections and nothing of the others', () => {
    const [argentina, portugal, japan] = holLocations(THREE);
    const pack = holPack([argentina!, japan!]);
    expect(pack).toBe(
      '[Argentina] 3\r\nSan Martín National Patriot Day,2026/8/17\r\nEid al Adha,1447/12/10,6\r\n' +
        "New Year's Day,2026/1/1\r\n[Japan] 1\r\nShōwa Day,2026/4/29\r\n\r\n",
    );
    expect(pack).not.toContain('Portugal');
    // What is sent is itself a pack with exactly the chosen locations.
    expect(holLocations(pack).map((l) => l.name)).toEqual(['Argentina', 'Japan']);
    expect(holPack([portugal!])).toBe('[Portugal] 1\r\nDia de Portugal,2026/6/10\r\n');
  });
});
