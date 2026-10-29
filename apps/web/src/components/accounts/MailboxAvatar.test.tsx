import { cleanup, render } from '@solidjs/testing-library';
import { afterEach, describe, expect, it, vi } from 'vitest';
vi.mock('../../i18n/index.ts', () => ({ t: (key: string) => key, isolate: (value: string) => value, loadCatalog: async () => undefined }));
import { MailboxAvatar, MailboxPicker, type MailboxChoice } from './MailboxPicker.tsx';

afterEach(cleanup);

describe('MailboxAvatar (t86)', () => {
  it('without a saved avatar it is the generated initials and colour class, no inline colours', () => {
    const { container } = render(() => <MailboxAvatar label="Work" address="work@example.org" />);
    const avatar = container.querySelector<HTMLElement>('.person-avatar')!;
    expect(avatar).toHaveTextContent('W');
    expect(avatar.dataset['avatarColor']).toMatch(/^\d+$/);
    expect(avatar.dataset['avatarKind']).toBeUndefined();
    expect(avatar.style.backgroundColor).toBe('');
    expect(avatar.getAttribute('aria-hidden')).toBe('true');
  });

  it('an icon avatar draws the glyph in the chosen colours', () => {
    const { container } = render(() => <MailboxAvatar label="Work" address="work@example.org" avatar={{ kind: 'icon', icon: 'office', fg: '#ffffff', bg: '#1d4ed8' }} />);
    const avatar = container.querySelector<HTMLElement>('.person-avatar')!;
    expect(avatar.dataset['avatarKind']).toBe('icon');
    expect(avatar.dataset['avatarIcon']).toBe('office');
    expect(avatar.querySelector('svg')?.getAttribute('data-icon')).toBe('office');
    expect(avatar.textContent).toBe('');
    expect(avatar.style.backgroundColor).toBe('rgb(29, 78, 216)');
    expect(avatar.style.color).toBe('rgb(255, 255, 255)');
    expect(avatar.dataset['avatarColor']).toBeUndefined();
  });

  it('a letter avatar shows the letter as typed; Auto picks a readable foreground', () => {
    const { container } = render(() => <MailboxAvatar label="Work" address="work@example.org" avatar={{ kind: 'letter', text: 'ab', bg: '#fde68a' }} />);
    const avatar = container.querySelector<HTMLElement>('.person-avatar')!;
    expect(avatar).toHaveTextContent('ab');
    expect(avatar.style.color).toBe('rgb(0, 0, 0)');
  });

  it('the picker trigger, entries and compact button all show the saved avatar', () => {
    const choices: MailboxChoice[] = [
      { linkId: null, address: 'me@example.org', signIn: true, available: true, unread: null, avatar: { kind: 'icon', icon: 'house', bg: '#14532d' } },
      { linkId: 'w', address: 'work@example.org', nickname: 'Work', available: true, unread: 2, avatar: { kind: 'letter', text: 'WK', fg: '#111111', bg: '#fde68a' } },
    ];
    const full = render(() => <MailboxPicker variant="full" choices={choices} activeLinkId="w" busy={false} onSelect={() => undefined} />);
    const trigger = full.container.querySelector<HTMLElement>('[data-account-indicator="picker"] .person-avatar')!;
    expect(trigger).toHaveTextContent('WK');
    full.container.querySelector<HTMLButtonElement>('[data-account-indicator="picker"]')!.click();
    const items = [...document.querySelectorAll<HTMLElement>('[data-mailbox-item] .person-avatar')];
    expect(items.map((item) => item.dataset['avatarIcon'] ?? item.textContent)).toEqual(['house', 'WK']);
    full.unmount();
    const icon = render(() => <MailboxPicker variant="icon" choices={choices} activeLinkId={null} busy={false} onSelect={() => undefined} />);
    expect(icon.container.querySelector<HTMLElement>('[data-account-indicator="icon"] .person-avatar')?.dataset['avatarIcon']).toBe('house');
  });
});
