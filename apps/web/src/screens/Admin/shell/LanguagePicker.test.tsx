import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, within } from '@solidjs/testing-library';
import { ADMIN_LOCALE_KEY, LanguagePicker, useStoredAdminLocale } from './LanguagePicker.tsx';
import { LOCALES, LocaleProvider } from '../../../i18n';
import { activeLocaleSignal, setActiveLocale } from '../../../i18n/registry.ts';

function Mounted() {
  useStoredAdminLocale();
  return <LanguagePicker />;
}

beforeEach(() => localStorage.clear());
afterEach(() => setActiveLocale('en'));

describe('LanguagePicker', () => {
  it('lists every shipped locale by its own name', () => {
    render(() => (
      <LocaleProvider locale="en">
        <LanguagePicker />
      </LocaleProvider>
    ));
    const select = screen.getByRole('combobox', { name: 'Language' });
    const options = within(select).getAllByRole('option');
    expect(options.map((o) => o.getAttribute('value'))).toEqual([...LOCALES]);
    expect(within(select).getByRole('option', { name: 'Deutsch' })).toHaveAttribute('lang', 'de');
    expect(select).toHaveValue('en');
  });

  it('switching sets the app locale and remembers the choice for the console', () => {
    render(() => (
      <LocaleProvider locale="en">
        <LanguagePicker />
      </LocaleProvider>
    ));
    fireEvent.change(screen.getByRole('combobox', { name: 'Language' }), { target: { value: 'de' } });
    expect(activeLocaleSignal()).toBe('de');
    expect(localStorage.getItem(ADMIN_LOCALE_KEY)).toBe('de');
  });

  it('a stored choice is applied when the console mounts; an unknown stored value is ignored', async () => {
    localStorage.setItem(ADMIN_LOCALE_KEY, 'fr');
    const first = render(() => (
      <LocaleProvider locale="en">
        <Mounted />
      </LocaleProvider>
    ));
    await Promise.resolve();
    expect(activeLocaleSignal()).toBe('fr');
    first.unmount();
    localStorage.setItem(ADMIN_LOCALE_KEY, 'xx-nope');
    render(() => (
      <LocaleProvider locale="en">
        <Mounted />
      </LocaleProvider>
    ));
    await Promise.resolve();
    expect(activeLocaleSignal()).toBe('en');
  });

  it('renders nothing outside a locale provider', () => {
    const { container } = render(() => <LanguagePicker />);
    expect(container).toBeEmptyDOMElement();
  });
});
