import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { clearSettings, loadSettings, type Fetcher } from '../../api/settings.ts';
import { SendDelayPreferences } from './SendDelayPreferences.tsx';

const fixture = vi.hoisted(() => ({ locked: false, value: 10, save: vi.fn(async (_value: number) => true) }));
vi.mock('./useSetting.ts', () => ({
  useSetting: () => ({
    value: () => fixture.value,
    entry: () => ({ value: fixture.value }),
    locked: () => fixture.locked,
    saving: () => false,
    error: () => '',
    reset: async () => true,
    set: (value: number) => fixture.save(value),
  }),
}));

const json = (body: unknown): Response => new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
async function load(values: Record<string, unknown>): Promise<void> {
  const settings = Object.fromEntries(Object.entries(values).map(([key, value]) => [key, { value, source: 'default', locked: false }]));
  await loadSettings({ refresh: true, fetcher: (async () => json({ settings })) as Fetcher });
}
afterEach(() => { cleanup(); clearSettings(); fixture.locked = false; fixture.value = 10; fixture.save.mockClear(); });

it('offers presets and stores the chosen seconds', () => {
  render(() => <SendDelayPreferences />);
  const select = screen.getByRole('combobox') as HTMLSelectElement;
  expect(select.value).toBe('10');
  fireEvent.change(select, { target: { value: '120' } });
  expect(fixture.save).toHaveBeenCalledWith(120);
  fireEvent.change(select, { target: { value: '0' } });
  expect(fixture.save).toHaveBeenCalledWith(0);
});

it('limits the choices to the administrator cap', async () => {
  await load({ 'mail.send.recallDelayMaxSeconds': 60 });
  render(() => <SendDelayPreferences />);
  const values = [...(screen.getByRole('combobox') as HTMLSelectElement).options].map((o) => o.value);
  expect(values).toEqual(['0', '5', '10', '20', '30', '60', 'custom']);
});

it('accepts a custom value only inside 0..cap', async () => {
  await load({ 'mail.send.recallDelayMaxSeconds': 100 });
  fixture.value = 45;
  render(() => <SendDelayPreferences />);
  const input = document.querySelector('input[type="number"]') as HTMLInputElement;
  expect(input.value).toBe('45');
  fireEvent.change(input, { target: { value: '500' } });
  expect(fixture.save).not.toHaveBeenCalled();
  expect(screen.getByRole('alert')).toBeTruthy();
  fireEvent.change(input, { target: { value: '90' } });
  expect(fixture.save).toHaveBeenCalledWith(90);
});

it('is disabled with a notice when the administrator turned recall off, or locked the value', async () => {
  await load({ 'mail.send.allowRecall': false });
  render(() => <SendDelayPreferences />);
  expect((screen.getByRole('combobox') as HTMLSelectElement).disabled).toBe(true);
  expect(screen.getByRole('status')).toBeTruthy();
  cleanup();
  clearSettings();
  fixture.locked = true;
  render(() => <SendDelayPreferences />);
  expect((screen.getByRole('combobox') as HTMLSelectElement).disabled).toBe(true);
});
