// Time-grid rendering (Outlook-style): concurrent events sit side by side,
// day-long events go to the all-day row, chips carry their status and an ink
// colour that reads on the calendar colour. Drives TimeGridView over a stub
// controller exposing only what the view reads.

import { describe, it, expect } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { TimeGridView } from './views.tsx';
import type { CalendarController } from './controller.ts';
import type { EventInstance } from './types.ts';
import type { CalendarEvent } from '../../api/pim-types.ts';

const DAY = new Date(2026, 9, 29);

function inst(id: string, title: string, start: Date, end: Date, over: Partial<CalendarEvent> = {}, color = '#1e3a8a'): EventInstance {
  const event = { id, calendarId: 'cal', title, status: 'confirmed', ...over } as CalendarEvent;
  return { key: `${id}:${start.getTime()}`, event, start, end, allDay: event.showWithoutTime === true, recurring: false, color };
}

function stub(list: EventInstance[]): CalendarController {
  return {
    view: () => 'day',
    focusDate: () => DAY,
    instancesForDay: (d: Date) => {
      const s = new Date(d.getFullYear(), d.getMonth(), d.getDate());
      const e = new Date(s.getFullYear(), s.getMonth(), s.getDate() + 1);
      return list.filter((i) => i.start < e && i.end > s);
    },
    hasConflict: () => false,
    goToDate: () => undefined,
  } as unknown as CalendarController;
}

const at = (h: number, m = 0): Date => new Date(2026, 9, 29, h, m);

describe('TimeGridView layout', () => {
  it('places overlapping events in side-by-side columns', () => {
    render(() => (
      <TimeGridView
        controller={stub([inst('a', 'Alpha', at(9), at(10)), inst('b', 'Beta', at(9, 30), at(11)), inst('c', 'Gamma', at(14), at(15))])}
        onOpenEvent={() => undefined}
      />
    ));
    const a = screen.getByTitle('Alpha');
    const b = screen.getByTitle('Beta');
    const c = screen.getByTitle('Gamma');
    expect(a.style.width).toBe('calc(50% - 4px)');
    expect(b.style.width).toBe('calc(50% - 4px)');
    expect(a.style.getPropertyValue('inset-inline-start')).toBe('calc(0% + 2px)');
    expect(b.style.getPropertyValue('inset-inline-start')).toBe('calc(50% + 2px)');
    expect(c.style.width).toBe('calc(100% - 4px)');
  });

  it('puts day-long timed events in the all-day row, not the hourly grid', () => {
    render(() => (
      <TimeGridView
        controller={stub([inst('m', 'Conference', at(8), new Date(2026, 9, 31, 8)), inst('t', 'Standup', at(9), at(9, 15))])}
        onOpenEvent={() => undefined}
      />
    ));
    const column = screen.getByTestId('day-column');
    expect(column.contains(screen.getByTitle('Conference'))).toBe(false);
    expect(column.contains(screen.getByTitle('Standup'))).toBe(true);
  });

  it('marks cancelled and tentative chips and picks a readable ink', () => {
    render(() => (
      <TimeGridView
        controller={stub([
          inst('x', 'Dropped', at(9), at(10), { status: 'cancelled' }),
          inst('y', 'Maybe', at(11), at(12), { status: 'tentative' }, '#ffeb3b'),
          inst('z', 'Firm', at(13), at(14)),
        ])}
        onOpenEvent={() => undefined}
      />
    ));
    expect(screen.getByTitle('Dropped').dataset['status']).toBe('cancelled');
    expect(screen.getByTitle('Maybe').dataset['status']).toBe('tentative');
    expect(screen.getByTitle('Firm').dataset['status']).toBeUndefined();
    // Dark ink on a light yellow calendar, light ink on navy.
    expect(screen.getByTitle('Maybe').style.color).toBe('rgb(17, 17, 17)');
    expect(screen.getByTitle('Firm').style.color).toBe('rgb(255, 255, 255)');
  });
});
