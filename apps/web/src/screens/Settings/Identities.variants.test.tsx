import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { Identities } from './Identities.tsx';
import { SettingsError, type SettingsService } from './service.ts';
import { inheritedFields, type Identity } from './types.ts';

beforeEach(() => activateAccountPreferences('identity-variants'));
afterEach(() => {
  cleanup();
  suspendAccountPreferences();
});

const base: Identity = { id: 'base', name: 'Alex', email: 'alex@example.org', replyTo: 'help@example.org', isDefault: true };
const variant: Identity = { id: 'news', name: 'Alex (news)', email: 'alex@example.org', variantOf: 'base', label: 'Newsletters', organization: 'Own Org' };
const other: Identity = { id: 'other', name: 'Sam', email: 'sam@example.org' };

function service(rows: Identity[], extra: Record<string, unknown> = {}): SettingsService {
  return { listIdentities: vi.fn(async () => rows), listSignatures: vi.fn(async () => []), ...extra } as unknown as SettingsService;
}

describe('Identities — default and variants', () => {
  it('lists variants under their base with a default badge and inherit hint', async () => {
    render(() => <Identities service={service([base, other, variant])} />);
    const items = await screen.findAllByRole('listitem');
    expect(items.map((item) => item.getAttribute('data-testid'))).toEqual(['identity-base', 'identity-variant', 'identity-base']);
    expect(within(items[0]!).getByTestId('identity-default-badge')).toHaveTextContent('Default');
    expect(within(items[1]!).getByTestId('identity-label')).toHaveTextContent('Newsletters');
    expect(within(items[1]!).getByTestId('identity-inherits')).toHaveTextContent('reply-to');
    expect(within(items[1]!).getByTestId('identity-inherits')).not.toHaveTextContent('organization');
    expect(within(items[1]!).queryByText('Create variant')).toBeNull();
    expect(within(items[0]!).queryByText('Make default')).toBeNull();
    expect(within(items[2]!).getByText('Make default')).toBeInTheDocument();
  });

  it('makes an identity the default', async () => {
    const makeIdentityDefault = vi.fn(async () => undefined);
    render(() => <Identities service={service([base, other], { makeIdentityDefault })} />);
    const row = (await screen.findAllByRole('listitem'))[1]!;
    fireEvent.click(within(row).getByText('Make default'));
    await waitFor(() => expect(makeIdentityDefault).toHaveBeenCalledWith('other'));
  });

  it('creates a variant prefilled from its base and requires a label', async () => {
    const upsert = vi.fn(async (_i: Identity) => undefined);
    render(() => <Identities service={service([base], { upsertIdentity: upsert })} />);
    fireEvent.click(await screen.findByText('Create variant'));
    expect(await screen.findByTestId('identity-variant-intro')).toBeInTheDocument();
    expect(screen.getByLabelText('Email address')).toHaveValue('alex@example.org');
    fireEvent.click(screen.getByTestId('identity-save'));
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(upsert).not.toHaveBeenCalled();
    fireEvent.input(screen.getByLabelText('Variant label'), { target: { value: 'Newsletters' } });
    fireEvent.click(screen.getByTestId('identity-save'));
    await waitFor(() => expect(upsert).toHaveBeenCalled());
    // The same address as its base is allowed for a variant.
    expect(upsert.mock.calls[0]![0]).toMatchObject({ id: '', variantOf: 'base', label: 'Newsletters', email: 'alex@example.org' });
  });

  it('asks what to do with variants before deleting a base', async () => {
    const deleteIdentity = vi.fn(async () => undefined);
    render(() => <Identities service={service([base, variant], { deleteIdentity })} />);
    const row = (await screen.findAllByRole('listitem'))[0]!;
    fireEvent.click(within(row).getByText('Delete'));
    expect(deleteIdentity).not.toHaveBeenCalled();
    fireEvent.click(await screen.findByTestId('identity-delete-keep-variants'));
    await waitFor(() => expect(deleteIdentity).toHaveBeenCalledWith('base', 'promote'));
    await waitFor(() => expect(screen.queryByTestId('identity-variants-choice')).toBeNull());
    await waitFor(() => expect(within(screen.getAllByRole('listitem')[0]!).getByText('Delete')).not.toBeDisabled());
    fireEvent.click(within(screen.getAllByRole('listitem')[0]!).getByText('Delete'));
    fireEvent.click(await screen.findByTestId('identity-delete-with-variants'));
    await waitFor(() => expect(deleteIdentity).toHaveBeenLastCalledWith('base', 'delete'));
  });

  it('falls back to the choice when the server reports variants it listed late', async () => {
    const deleteIdentity = vi.fn(async () => {
      throw new SettingsError(409, 'conflict', { code: 'has_variants' });
    });
    render(() => <Identities service={service([base], { deleteIdentity })} />);
    fireEvent.click(await screen.findByText('Delete'));
    fireEvent.click(within(await screen.findByTestId('identity-delete-confirm')).getByText('Delete'));
    await waitFor(() => expect(screen.getByTestId('identity-variants-choice')).toBeInTheDocument());
  });
});

describe('inheritedFields', () => {
  it('lists the empty inheritable fields and treats an empty signature name as an own value', () => {
    expect(inheritedFields({ id: 'v', name: 'V', email: 'v@x.y', variantOf: 'b', label: 'L' })).toEqual(['replyTo', 'autoBcc', 'organization', 'pgpKeyId', 'signatureName']);
    expect(inheritedFields({ id: 'v', name: 'V', email: 'v@x.y', variantOf: 'b', replyTo: 'r@x.y', signatureName: '' })).toEqual(['autoBcc', 'organization', 'pgpKeyId']);
  });
});
