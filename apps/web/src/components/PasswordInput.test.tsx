import { describe, it, expect, vi } from 'vitest';
import { createSignal, Show } from 'solid-js';
import { render, screen, fireEvent } from '@solidjs/testing-library';
import { PasswordInput } from './PasswordInput.tsx';

function field(): HTMLInputElement {
  return screen.getByRole('textbox', { name: 'Password' }) as HTMLInputElement;
}

function hiddenField(): HTMLInputElement {
  // A password-type input has no ARIA role; find it by its name instead.
  return screen.getByLabelText('Password') as HTMLInputElement;
}

describe('PasswordInput', () => {
  it('starts hidden, with a toggle that names and controls the field', () => {
    render(() => (
      <label>
        <span>Password</span>
        <PasswordInput label="Password" autocomplete="current-password" name="pw" />
      </label>
    ));
    const input = hiddenField();
    expect(input.type).toBe('password');
    expect(input.getAttribute('autocomplete')).toBe('current-password');
    expect(input.name).toBe('pw');
    const toggle = screen.getByRole('button', { name: 'Show password' });
    expect(toggle.getAttribute('type')).toBe('button');
    expect(toggle.getAttribute('aria-pressed')).toBe('false');
    expect(toggle.getAttribute('aria-controls')).toBe(input.id);
    expect(input.id).not.toBe('');
  });

  it('the field keeps a clean name: the toggle inside its <label> is not part of it', () => {
    render(() => (
      <label>
        <span>Password</span>
        <PasswordInput label="Password" />
      </label>
    ));
    expect(hiddenField().getAttribute('aria-label')).toBe('Password');
  });

  it('shows and hides the typed text; the name stays, aria-pressed follows', () => {
    render(() => <PasswordInput label="Password" value="hunter2" />);
    const toggle = screen.getByRole('button', { name: 'Show password' });
    fireEvent.click(toggle);
    expect(field().type).toBe('text');
    expect(field().value).toBe('hunter2');
    expect(toggle.getAttribute('aria-pressed')).toBe('true');
    expect(toggle.getAttribute('title')).toBe('Hide password');
    // Shown as text: no capitals, corrections or spell check on a phone keyboard.
    expect(field().getAttribute('autocapitalize')).toBe('off');
    expect(field().getAttribute('spellcheck')).toBe('false');
    fireEvent.click(screen.getByRole('button', { name: 'Show password' }));
    expect(hiddenField().type).toBe('password');
    expect(toggle.getAttribute('aria-pressed')).toBe('false');
  });

  it('keeps an id the caller gives and points the toggle at it', () => {
    render(() => <PasswordInput label="Password" id="my-pw" />);
    expect(hiddenField().id).toBe('my-pw');
    expect(screen.getByRole('button', { name: 'Show password' }).getAttribute('aria-controls')).toBe('my-pw');
  });

  it('does not submit its form, and goes back to hidden when the form is submitted', () => {
    const submitted = vi.fn((e: Event) => e.preventDefault());
    render(() => (
      <form onSubmit={submitted}>
        <PasswordInput label="Password" />
        <button type="submit">Sign in</button>
      </form>
    ));
    fireEvent.click(screen.getByRole('button', { name: 'Show password' }));
    expect(submitted).not.toHaveBeenCalled();
    expect(field().type).toBe('text');
    fireEvent.submit(field().form!);
    expect(submitted).toHaveBeenCalledTimes(1);
    expect(hiddenField().type).toBe('password');
  });

  it('goes back to hidden when hideWhen changes (a failed sign-in)', () => {
    const [error, setError] = createSignal<string | null>(null);
    render(() => <PasswordInput label="Password" hideWhen={error} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show password' }));
    expect(field().type).toBe('text');
    setError('Wrong password');
    expect(hiddenField().type).toBe('password');
  });

  it('a press on the toggle does not take focus from the field, and the caret stays', () => {
    render(() => <PasswordInput label="Password" value="abcdef" />);
    const input = hiddenField();
    input.focus();
    input.setSelectionRange(2, 2);
    const toggle = screen.getByRole('button', { name: 'Show password' });
    const down = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
    toggle.dispatchEvent(down);
    expect(down.defaultPrevented).toBe(true);
    fireEvent.click(toggle);
    expect(document.activeElement).toBe(input);
    expect([input.selectionStart, input.selectionEnd]).toEqual([2, 2]);
  });

  it('starts hidden again after it unmounts and mounts', () => {
    const [on, setOn] = createSignal(true);
    render(() => (
      <Show when={on()}>
        <PasswordInput label="Password" />
      </Show>
    ));
    fireEvent.click(screen.getByRole('button', { name: 'Show password' }));
    expect(field().type).toBe('text');
    setOn(false);
    setOn(true);
    expect(hiddenField().type).toBe('password');
  });

  it('is disabled with its field', () => {
    render(() => <PasswordInput label="Password" disabled />);
    expect((screen.getByRole('button', { name: 'Show password' }) as HTMLButtonElement).disabled).toBe(true);
  });
});
