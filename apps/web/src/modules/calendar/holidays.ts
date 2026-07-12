// Bundled holiday packs (plan §3 e4: "holiday subscribe"; §11 bundled `.hol`
// packs + per-region subscribe). Each pack is a small ICS blob the module
// imports into the chosen calendar via `CalendarEvent/import`; the engine (e8)
// ships the full region set + `.hol` conversion, so this is a minimal seed that
// keeps the subscribe UI functional against the mock until e10.

export interface HolidayPack {
  id: string;
  label: string;
  ics: string;
}

function pack(id: string, label: string, events: Array<{ uid: string; date: string; name: string }>): HolidayPack {
  const body = events
    .map((e) => `BEGIN:VEVENT\nUID:${e.uid}\nDTSTART;VALUE=DATE:${e.date.replace(/-/g, '')}\nSUMMARY:${e.name}\nEND:VEVENT`)
    .join('\n');
  return { id, label, ics: `BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//Mailwoman//Holidays//EN\n${body}\nEND:VCALENDAR` };
}

export const HOLIDAY_PACKS: readonly HolidayPack[] = [
  pack('us', 'United States', [
    { uid: 'us-newyear-2026', date: '2026-01-01', name: "New Year's Day" },
    { uid: 'us-independence-2026', date: '2026-07-04', name: 'Independence Day' },
    { uid: 'us-thanksgiving-2026', date: '2026-11-26', name: 'Thanksgiving' },
    { uid: 'us-christmas-2026', date: '2026-12-25', name: 'Christmas Day' },
  ]),
  pack('uk', 'United Kingdom', [
    { uid: 'uk-newyear-2026', date: '2026-01-01', name: "New Year's Day" },
    { uid: 'uk-mayday-2026', date: '2026-05-04', name: 'Early May Bank Holiday' },
    { uid: 'uk-christmas-2026', date: '2026-12-25', name: 'Christmas Day' },
    { uid: 'uk-boxing-2026', date: '2026-12-26', name: 'Boxing Day' },
  ]),
  pack('pt', 'Portugal', [
    { uid: 'pt-newyear-2026', date: '2026-01-01', name: 'Ano Novo' },
    { uid: 'pt-liberdade-2026', date: '2026-04-25', name: 'Dia da Liberdade' },
    { uid: 'pt-natal-2026', date: '2026-12-25', name: 'Natal' },
  ]),
];
