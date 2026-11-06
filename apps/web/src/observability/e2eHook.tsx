// A test-only way to push a deliberate error through a REAL error boundary so the
// whole client-error path (boundary -> reporter -> server -> Admin > Observability
// > Client errors, with the symbolicated stack) can be exercised in a browser.
//
// It exists only in a build made with `VITE_MW_E2E_HOOKS=1`; a normal build drops
// the dynamic import (see main.tsx), so production bundles carry neither this
// file nor `window.__mwThrow`.

import { render } from 'solid-js/web';
import { AsyncBoundary } from '../components/ErrorBoundary.tsx';

export const E2E_BOUNDARY = 'E2E/DeliberateBoundary';

function Thrower(props: { message: string }): never {
  throw new Error(props.message);
}

declare global {
  interface Window {
    __mwThrow?: (message?: string) => void;
  }
}

/** `window.__mwThrow(message)` renders a throwing component inside an `AsyncBoundary` named {@link E2E_BOUNDARY}. */
export function installE2eErrorHook(): void {
  if (import.meta.env['VITE_MW_E2E_HOOKS'] !== '1') return;
  window.__mwThrow = (message = 'Deliberate boundary error (e2e)') => {
    const host = document.createElement('div');
    host.dataset['testid'] = 'e2e-boundary-host';
    document.body.appendChild(host);
    render(() => (
      <AsyncBoundary name={E2E_BOUNDARY}>
        <Thrower message={message} />
      </AsyncBoundary>
    ), host);
  };
}
