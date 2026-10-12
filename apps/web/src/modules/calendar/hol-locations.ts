// The locations of a `.hol` holiday pack, for the choice the import asks for.
//
// A pack is a run of sections: a `[Name] <n>` line, then that location's
// holiday lines. Outlook's own OUTLOOK.HOL has 116 sections and about 23,000
// dated lines, and the engine imports every section of the text it is sent
// (`parse_hol`, crates/mw-ics/src/hol.rs). So a pack with more than one
// location is never sent whole: the user picks, and only the picked sections
// are sent (`CalendarApp`, index.tsx).
//
// What is read here is the section rule alone, the engine's `is_section_header`
// (hol.rs): a line that, trimmed, opens with `[`, has a `]` after it and a
// non-empty name between the two. Which of a section's lines are holidays is
// the engine's decision and is not repeated here, so no holiday count is shown
// before the import; the count the user sees is the engine's answer.

export interface HolLocation {
  /** The text between the brackets, trimmed. */
  name: string;
  /** The section as it stands in the file: its header line, then the lines under it. */
  lines: string[];
}

/** The name in a section header line, or `null` when the line is not one. */
function sectionName(line: string): string | null {
  const l = line.trim();
  if (!l.startsWith('[')) return null;
  const close = l.indexOf(']');
  if (close < 0) return null;
  const name = l.slice(1, close).trim();
  return name === '' ? null : name;
}

/**
 * The locations of `text`, in file order. Empty when `text` holds
 * `BEGIN:VCALENDAR` — the engine reads such a blob as iCalendar, never as a
 * pack (`parse_calendar_blob`, events.rs) — or when it has no section header.
 * Lines before the first header belong to no location; the engine does not
 * read them either.
 */
export function holLocations(text: string): HolLocation[] {
  if (text.includes('BEGIN:VCALENDAR')) return [];
  const out: HolLocation[] = [];
  for (const line of text.split(/\r?\n/)) {
    const name = sectionName(line);
    if (name !== null) out.push({ name, lines: [line] });
    else out[out.length - 1]?.lines.push(line);
  }
  return out;
}

/** A pack that holds `chosen` and nothing else, as the text to send. */
export function holPack(chosen: readonly HolLocation[]): string {
  return chosen.map((location) => `${location.lines.join('\r\n')}\r\n`).join('');
}
