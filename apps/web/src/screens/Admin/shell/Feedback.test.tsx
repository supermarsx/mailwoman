import { describe, it, expect } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { Feedback, describeAdminError } from './Feedback.tsx';
import { AdminApiError } from '../../../state/slices/admin.ts';
import { ToastRegion, toasts } from '../../../components/toast/index.ts';

describe('Feedback', () => {
  it('an inline failure is an assertive alert and an inline note a polite status, each with an icon', () => {
    render(() => (
      <>
        <Feedback kind="info" message="Set by the environment." />
        <Feedback kind="error" message="Could not load." />
      </>
    ));
    expect(screen.getByRole('status')).toHaveTextContent('Set by the environment.');
    expect(screen.getByRole('alert')).toHaveTextContent('Could not load.');
    expect(screen.getByRole('status').querySelector('svg')).toHaveAttribute('data-icon', 'info');
    expect(screen.getByRole('alert').querySelector('svg')).toHaveAttribute('data-icon', 'alert');
  });

  it('a success is a toast, and nothing stays in the page', () => {
    const { container } = render(() => (
      <>
        <div data-testid="page">
          <Feedback kind="success" message="Saved." testId="saved-notice" />
        </div>
        <ToastRegion />
      </>
    ));
    expect(container.querySelector('[data-testid="page"]')).toBeEmptyDOMElement();
    expect(toasts().map((i) => [i.tone, i.message, i.testId])).toEqual([['success', 'Saved.', 'saved-notice']]);
    expect(screen.getByTestId('saved-notice')).toHaveAttribute('role', 'status');
  });

  it('stays inline while no toast region is mounted', () => {
    render(() => <Feedback kind="success" message="Saved." />);
    expect(screen.getByRole('status')).toHaveTextContent('Saved.');
    expect(toasts()).toEqual([]);
  });

  it('as="toast" sends a failure to the toasts, and a new result replaces the last one', () => {
    const [msg, setMsg] = createSignal<string | null>(null);
    render(() => (
      <>
        <Feedback kind="error" as="toast" message={msg()} />
        <ToastRegion />
      </>
    ));
    expect(screen.queryByRole('alert')).toBeNull();
    setMsg('Could not save.');
    expect(screen.getByRole('alert')).toHaveTextContent('Could not save.');
    setMsg(null);
    setMsg('Still could not save.');
    expect(screen.getAllByRole('alert').map((n) => n.textContent)).toEqual(['Still could not save.']);
  });

  it('renders nothing without a message', () => {
    const { container } = render(() => <Feedback kind="error" message={null} />);
    expect(container).toBeEmptyDOMElement();
  });
});

describe('describeAdminError', () => {
  it('maps a server code to its message with a next step', () => {
    expect(describeAdminError({ status: 502, code: 'upstream_tls_failed' })).toBe(
      'The server answered but the TLS handshake failed. Check the port and the connection security setting.',
    );
  });

  it('falls back on the HTTP status', () => {
    expect(describeAdminError(new AdminApiError(403, 'x'))).toMatch(/^Your role does not allow this action/);
    expect(describeAdminError(new AdminApiError(409, 'x'))).toMatch(/changed it in the meantime/);
    expect(describeAdminError({ status: 412, code: 'stale' })).toMatch(/changed it in the meantime/);
    expect(describeAdminError(new AdminApiError(429, 'x'))).toMatch(/Too many requests/);
    expect(describeAdminError(new AdminApiError(503, 'x'))).toBe(
      'The server answered with an error (503). Try again; if it repeats, check the server log.',
    );
  });

  it('a thrown network error says the server could not be reached', () => {
    expect(describeAdminError(new TypeError('Failed to fetch'))).toMatch(/could not be reached/);
  });

  it('an unknown code falls back to the status', () => {
    expect(describeAdminError({ status: 404, code: 'something_new' })).toMatch(/no longer exists/);
  });
});
