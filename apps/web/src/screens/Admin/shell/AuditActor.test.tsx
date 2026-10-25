import { describe, expect, it } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { AuditActor } from './AuditActor.tsx';

describe('audit actor presentation', () => {
  it('shows the resolved identity and links to its permitted detail page', () => {
    render(() => <AuditActor actor="opaque-hash" actorLabel="Alice (alice@example.test)" actorLink={{section:'users',id:'opaque-hash'}} />);
    expect(screen.getByRole('link', {name:'Alice (alice@example.test)'})).toHaveAttribute('href', expect.stringContaining('users/opaque-hash'));
    expect(screen.queryByText('opaque-hash')).not.toBeInTheDocument();
  });
  it('keeps unresolved identifiers visible without inventing a link', () => {
    render(() => <AuditActor actor="unknown-actor" />);
    expect(screen.getByText('unknown-actor')).toHaveAttribute('title','unknown-actor');
    expect(screen.queryByRole('link')).not.toBeInTheDocument();
  });
});
