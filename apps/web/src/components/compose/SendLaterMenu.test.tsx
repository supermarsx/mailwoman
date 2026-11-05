import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { SendLaterMenu } from './SendLaterMenu.tsx';

afterEach(() => { cleanup(); vi.useRealTimers(); });

describe('SendLaterMenu', () => {
  it('lists presets and passes the picked local time through', () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date(2026, 0, 7, 9, 30));
    const onPick = vi.fn();
    render(() => <SendLaterMenu onPick={onPick} />);
    fireEvent.click(document.querySelector('[aria-haspopup="menu"]')!);
    expect(document.querySelectorAll('[data-preset]').length).toBeGreaterThanOrEqual(4);
    fireEvent.click(document.querySelector('[data-preset="tomorrow"]')!);
    expect(onPick).toHaveBeenCalledWith('2026-01-08T09:00');
  });

  it('refuses a custom time in the past and shows why', () => {
    const onPick = vi.fn();
    render(() => <SendLaterMenu onPick={onPick} />);
    fireEvent.click(document.querySelector('[aria-haspopup="menu"]')!);
    fireEvent.click(document.querySelector('[data-preset="custom"]')!);
    const input = document.querySelector('input[type="datetime-local"]') as HTMLInputElement;
    fireEvent.input(input, { target: { value: '2001-01-01T00:00' } });
    fireEvent.click(screen.getByTestId('send-later-schedule'));
    expect(onPick).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toBeTruthy();
  });

  it('accepts a future custom time and shows the time zone', () => {
    const onPick = vi.fn();
    render(() => <SendLaterMenu onPick={onPick} />);
    fireEvent.click(document.querySelector('[aria-haspopup="menu"]')!);
    fireEvent.click(document.querySelector('[data-preset="custom"]')!);
    expect(document.querySelector('.send-later__zone')).toBeTruthy();
    const future = new Date(Date.now() + 5 * 86_400_000);
    const p = (n: number): string => String(n).padStart(2, '0');
    const value = `${future.getFullYear()}-${p(future.getMonth() + 1)}-${p(future.getDate())}T${p(future.getHours())}:00`;
    fireEvent.input(document.querySelector('input[type="datetime-local"]')!, { target: { value } });
    fireEvent.click(screen.getByTestId('send-later-schedule'));
    expect(onPick).toHaveBeenCalledWith(value);
  });
});
