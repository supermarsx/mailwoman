import {afterEach, expect, it, vi} from 'vitest';
import {cleanup, fireEvent, render, screen, within} from '@solidjs/testing-library';
import {readFileSync} from 'node:fs';
import {FluentBundle, FluentResource} from '@fluent/bundle';
import {BarPreferences} from './BarPreferences.tsx';
import {BAR_IDS, barSettingKey, isBarMode} from '../../api/barSettings.ts';

const fixture = vi.hoisted(() => ({
  values: {} as Record<string, unknown>,
  locked: new Set<string>(),
  errors: {} as Record<string, string>,
  known: true,
  save: vi.fn(async (_key: string, _value: unknown) => true),
}));

vi.mock('./useSetting.ts', () => ({
  useSetting: (key: string, fallback: unknown, accept: (v: unknown) => boolean) => ({
    value: () => (accept(fixture.values[key]) ? fixture.values[key] : fallback),
    entry: () => (fixture.known ? {value: fixture.values[key] ?? fallback, source: 'default', locked: fixture.locked.has(key)} : undefined),
    locked: () => fixture.locked.has(key),
    saving: () => false,
    error: () => fixture.errors[key] ?? '',
    reset: async () => true,
    set: (value: unknown) => fixture.save(key, value),
  }),
}));

afterEach(() => {
  cleanup();
  fixture.values = {};
  fixture.locked.clear();
  fixture.errors = {};
  fixture.known = true;
  fixture.save.mockClear();
});

const group = (name: string): HTMLElement => screen.getByRole('group', {name});

it('starts every bar pinned and saves the choice to its registry key', () => {
  render(() => <BarPreferences />);
  for (const name of ['Mail toolbar', 'Workspace tabs', 'Mail ribbon', 'Editor ribbon']) {
    expect(within(group(name)).getByLabelText('Always shown')).toBeChecked();
    expect(within(group(name)).getByLabelText('Auto-hide')).not.toBeChecked();
  }
  fireEvent.click(within(group('Editor ribbon')).getByLabelText('Auto-hide'));
  expect(fixture.save).toHaveBeenCalledWith('ui.bar.composerRibbon', 'auto');
  fireEvent.click(within(group('Mail toolbar')).getByLabelText('Auto-hide'));
  expect(fixture.save).toHaveBeenCalledWith('ui.bar.mailTopBar', 'auto');
});

it('shows the stored value and ignores a malformed one', () => {
  fixture.values = {'ui.bar.workspaceTabs': 'auto', 'ui.bar.mailRibbon': 'sideways'};
  render(() => <BarPreferences />);
  expect(within(group('Workspace tabs')).getByLabelText('Auto-hide')).toBeChecked();
  expect(within(group('Mail ribbon')).getByLabelText('Always shown')).toBeChecked();
});

it('holds a locked bar with the administrator note and leaves the others editable', () => {
  fixture.locked.add('ui.bar.mailRibbon');
  fixture.values = {'ui.bar.mailRibbon': 'auto'};
  render(() => <BarPreferences />);
  const ribbon = group('Mail ribbon');
  expect(within(ribbon).getByLabelText('Auto-hide')).toBeChecked();
  expect(within(ribbon).getByLabelText('Auto-hide')).toBeDisabled();
  expect(within(ribbon).getByLabelText('Always shown')).toBeDisabled();
  const note = within(ribbon).getByTestId('setting-lock-note');
  expect(ribbon.getAttribute('aria-describedby')).toContain(note.id);
  expect(screen.getAllByTestId('setting-lock-note')).toHaveLength(1);
  expect(within(group('Editor ribbon')).getByLabelText('Auto-hide')).toBeEnabled();
});

it('reports a failed save next to its bar', () => {
  fixture.errors = {'ui.bar.workspaceTabs': 'Could not save the setting.'};
  render(() => <BarPreferences />);
  const alert = within(group('Workspace tabs')).getByRole('alert');
  expect(alert).toHaveTextContent('Could not save the setting.');
  expect(group('Workspace tabs').getAttribute('aria-describedby')).toContain(alert.id);
  expect(screen.getAllByRole('alert')).toHaveLength(1);
});

it('keeps the choices inert until the server knows the keys', () => {
  fixture.known = false;
  render(() => <BarPreferences />);
  for (const input of screen.getAllByRole('radio')) expect(input).toBeDisabled();
});

it('maps every bar to a ui.bar registry key with an English label', () => {
  const bundle = new FluentBundle('en');
  expect(bundle.addResource(new FluentResource(readFileSync('locales/en/bars.ftl', 'utf8')))).toEqual([]);
  expect(BAR_IDS.map(barSettingKey)).toEqual(['ui.bar.mailTopBar', 'ui.bar.workspaceTabs', 'ui.bar.mailRibbon', 'ui.bar.composerRibbon']);
  for (const id of ['settings-bars-heading', 'settings-bars-help', 'settings-bars-mode-pinned', 'settings-bars-mode-auto', 'settings-bars-mail-top-bar', 'settings-bars-workspace-tabs', 'settings-bars-mail-ribbon', 'settings-bars-composer-ribbon']) {
    expect(bundle.getMessage(id)?.value, id).toBeTruthy();
  }
  expect([isBarMode('pinned'), isBarMode('auto'), isBarMode('hidden'), isBarMode(null)]).toEqual([true, true, false, false]);
});
