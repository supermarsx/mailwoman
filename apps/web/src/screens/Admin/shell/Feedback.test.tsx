import { describe, it, expect } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { Feedback, describeAdminError } from './Feedback.tsx';
import { AdminApiError } from '../../../state/slices/admin.ts';

describe('Feedback', () => {
  it('success is a polite status, failure an assertive alert, each with an icon', () => {
    render(() => (
      <>
        <Feedback kind="success" message="Saved." />
        <Feedback kind="error" message="Could not save." />
      </>
    ));
    expect(screen.getByRole('status')).toHaveTextContent('Saved.');
    expect(screen.getByRole('alert')).toHaveTextContent('Could not save.');
    expect(screen.getByRole('status').querySelector('svg')).toHaveAttribute('data-icon', 'check');
    expect(screen.getByRole('alert').querySelector('svg')).toHaveAttribute('data-icon', 'alert');
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
