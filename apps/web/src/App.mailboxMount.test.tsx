// t51: MailboxMount (the boot-splash hand-over, t52) must render the mailbox
// screen as a component. Called as a plain function inside its JSX, the screen
// body ran in a tracking scope: a signal it read at its top level re-ran it and
// remounted the whole mailbox, so a clicked list row was disconnected under the
// click and no message opened.
import { afterEach, expect, it } from 'vitest';
import { cleanup, fireEvent, render } from '@solidjs/testing-library';
import { createSignal, type JSX } from 'solid-js';
import { MailboxMount } from './App.tsx';

afterEach(cleanup);

it('a signal read at the screen top level does not remount it; a clicked row stays connected', () => {
  const [active, setActive] = createSignal('mail');
  let runs = 0;
  function Screen(): JSX.Element {
    runs += 1;
    const current = active(); // a top-level read, as MailboxScreen reads its tabs
    return <div data-initial={current}><button type="button" class="list__row" onPointerDown={() => setActive('message:a')}>row</button><span data-testid="active">{active()}</span></div>;
  }
  const { getByRole, getByTestId } = render(() => <MailboxMount screen={Screen} />);
  const row = getByRole('button', { name: 'row' });
  row.focus();
  fireEvent.pointerDown(row);
  expect(getByTestId('active').textContent).toBe('message:a');
  expect(runs).toBe(1);
  expect(row.isConnected).toBe(true);
  expect(document.activeElement).toBe(row);
});
