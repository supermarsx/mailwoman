import { expect, it } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { Combobox, filterComboboxOptions, type ComboboxOption } from './Combobox.tsx';
import { TimeZoneCombobox, timeZoneOptions } from './TimeZoneCombobox.tsx';
import { formatUtcOffset, zoneOffsetMinutes } from '../i18n/timezones.ts';

const options: ComboboxOption[] = [
  { value: 'a', label: 'Alpha', group: 'First' },
  { value: 'b', label: 'Beta', detail: 'second', group: 'First' },
  { value: 'c', label: 'Gamma', keywords: 'hidden-word', group: 'Second' },
];

it('filters by every typed word over label, detail and keywords', () => {
  expect(filterComboboxOptions(options, 'hidden').map((o) => o.value)).toEqual(['c']);
  expect(filterComboboxOptions(options, 'be sec').map((o) => o.value)).toEqual(['b']);
  expect(filterComboboxOptions(options, '')).toHaveLength(3);
});

it('lists groups with headings, moves with the keyboard and selects with Enter', () => {
  const [value, setValue] = createSignal('a');
  render(() => <form><Combobox label="Pick" value={value()} options={options} onChange={setValue} /></form>);
  const input = screen.getByRole('combobox');
  fireEvent.focus(input);
  expect(screen.getAllByRole('group')).toHaveLength(2);
  expect(screen.getByText('Second')).toBeTruthy();
  fireEvent.keyDown(input, { key: 'ArrowDown' });
  fireEvent.keyDown(input, { key: 'ArrowDown' });
  expect(input.getAttribute('aria-activedescendant')).toBe(screen.getAllByRole('option')[2]!.id);
  fireEvent.keyDown(input, { key: 'Enter' });
  expect(value()).toBe('c');
  expect(screen.queryByRole('listbox')).toBeNull();
});

it('Escape closes, typing filters and shows an empty state', () => {
  render(() => <Combobox label="Pick" value="a" options={options} onChange={() => {}} />);
  const input = screen.getByRole('combobox');
  fireEvent.focus(input);
  fireEvent.keyDown(input, { key: 'Escape' });
  expect(screen.queryByRole('listbox')).toBeNull();
  fireEvent.input(input, { target: { value: 'zzz' } });
  expect(screen.getByRole('status').textContent).toBe('No matches');
  fireEvent.input(input, { target: { value: 'gam' } });
  expect(screen.getAllByRole('option')).toHaveLength(1);
});

it('marks the stored value selected and the keyboard row active, separately', () => {
  render(() => <Combobox label="Pick" value="b" options={options} onChange={() => {}} />);
  fireEvent.focus(screen.getByRole('combobox'));
  const rows = screen.getAllByRole('option');
  expect(rows[1]!.getAttribute('aria-selected')).toBe('true');
  expect(rows[1]!.getAttribute('data-active')).toBe('true');
  expect(rows[0]!.getAttribute('aria-selected')).toBe('false');
});

it('computes DST-aware offsets and formats them', () => {
  const winter = new Date('2026-01-15T12:00:00Z'), summer = new Date('2026-07-15T12:00:00Z');
  expect(zoneOffsetMinutes('Europe/Lisbon', winter)).toBe(0);
  expect(zoneOffsetMinutes('Europe/Lisbon', summer)).toBe(60);
  expect(zoneOffsetMinutes('Asia/Kolkata', winter)).toBe(330);
  expect(zoneOffsetMinutes('America/St_Johns', winter)).toBe(-210);
  expect(zoneOffsetMinutes('Not/AZone', winter)).toBeNull();
  expect(formatUtcOffset(330)).toBe('UTC+05:30');
  expect(formatUtcOffset(-210)).toBe('UTC-03:30');
  expect(formatUtcOffset(0)).toBe('UTC');
});

it('time zone rows carry region group, offset and local time, and are found by GMT+1 or the city', () => {
  const summer = new Date('2026-07-15T12:00:00Z');
  const rows = timeZoneOptions(summer, ['Europe/Lisbon', 'America/New_York', 'UTC']);
  const lisbon = rows.find((r) => r.value === 'Europe/Lisbon')!;
  expect(lisbon.group).toBe('Europe');
  expect(lisbon.detail).toBe('UTC+01:00 · 13:00');
  expect(filterComboboxOptions(rows, 'lisbon').map((r) => r.value)).toEqual(['Europe/Lisbon']);
  expect(filterComboboxOptions(rows, 'Europe/Lisbon').map((r) => r.value)).toEqual(['Europe/Lisbon']);
  expect(filterComboboxOptions(rows, 'GMT+1').map((r) => r.value)).toContain('Europe/Lisbon');
  expect(filterComboboxOptions(rows, 'new york')[0]!.group).toBe('America');
});

it('pins the browser time zone on top and stores the requested value for it', () => {
  const browser = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
  const [value, setValue] = createSignal('Asia/Tokyo');
  render(() => <TimeZoneCombobox label="Zone" value={value()} browserValue="local" onChange={setValue} />);
  const input = screen.getByRole('combobox');
  fireEvent.focus(input);
  fireEvent.input(input, { target: { value: '' } });
  const first = screen.getAllByRole('option')[0]!;
  expect(first.textContent).toContain(`Use browser time zone (${browser})`);
  fireEvent.click(first);
  expect(value()).toBe('local');
});
