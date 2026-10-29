import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { afterEach, describe, expect, it, vi } from 'vitest';
vi.mock('../../i18n/index.ts', () => ({
  t: (key: string, args?: Record<string, string | number>) => (args?.['name'] !== undefined ? `${key}:${args['name']}` : args?.['ratio'] !== undefined ? `${key}:${args['ratio']}` : key),
  isolate: (value: string) => value,
  loadCatalog: async () => undefined,
}));
import { MailboxAvatarEditor } from './MailboxAvatarEditor.tsx';
import { LinkedAccountList } from './LinkedAccountList.tsx';
import type { MailboxAvatarSpec } from '../../api/mailboxAvatar.ts';
import type { LinkedMailboxList } from '../../api/linkedAccounts.ts';

afterEach(cleanup);
const flush = async (): Promise<void> => { for (let i = 0; i < 6; i++) await Promise.resolve(); };

function mount(avatar?: MailboxAvatarSpec, onSave = vi.fn(async () => true)) {
  render(() => <MailboxAvatarEditor mailboxKey="main" label="Work" address="work@example.org" avatar={avatar} onSave={onSave} />);
  const trigger = screen.getByRole('button', { name: 'linked-accounts-avatar-change-for:Work (work@example.org)' });
  return { trigger, onSave };
}
const editor = (): HTMLElement => document.querySelector<HTMLElement>('[data-avatar-editor="main"]')!;

describe('mailbox avatar editor (t86)', () => {
  it('the avatar is an icon button with a tooltip that opens a labelled dialog', () => {
    const { trigger } = mount();
    expect(trigger).toHaveAttribute('data-tooltip', 'linked-accounts-avatar-change');
    expect(trigger).toHaveAttribute('aria-expanded', 'false');
    expect(editor()).toHaveAttribute('hidden');
    fireEvent.click(trigger);
    expect(trigger).toHaveAttribute('aria-expanded', 'true');
    expect(editor()).not.toHaveAttribute('hidden');
    expect(screen.getByRole('dialog', { name: 'linked-accounts-avatar-title' })).toBe(editor());
    expect(editor().querySelector('[data-avatar-mode="letter"]')).toHaveAttribute('aria-pressed', 'true');
  });

  it('picks an icon from the grid (tooltips name each icon, arrows move) and saves it', async () => {
    const { trigger, onSave } = mount();
    fireEvent.click(trigger);
    fireEvent.click(editor().querySelector('[data-avatar-mode="icon"]')!);
    const radios = [...editor().querySelectorAll<HTMLButtonElement>('[role="radio"]')];
    expect(radios).toHaveLength(26);
    for (const radio of radios) {
      expect(radio.getAttribute('data-tooltip')).toMatch(/^linked-accounts-avatar-icon-/);
      expect(radio).toHaveAccessibleName(radio.getAttribute('data-tooltip')!);
    }
    // Roving focus: only the chosen icon is in the tab order.
    expect(radios.filter((radio) => radio.tabIndex === 0)).toHaveLength(1);
    fireEvent.keyDown(radios[0]!, { key: 'ArrowRight' });
    expect(radios[1]).toHaveAttribute('aria-checked', 'true');
    fireEvent.click(editor().querySelector('[data-avatar-choice="crown"]')!);
    expect(editor().querySelector('[data-avatar-preview] .person-avatar')).toHaveAttribute('data-avatar-icon', 'crown');
    fireEvent.click(editor().querySelector('[data-avatar-save]')!);
    await flush();
    expect(onSave).toHaveBeenCalledWith({ kind: 'icon', icon: 'crown' });
    expect(editor()).toHaveAttribute('hidden');
  });

  it('a letter is checked as typed; one or two characters save, more are refused', async () => {
    const { trigger, onSave } = mount();
    fireEvent.click(trigger);
    const input = editor().querySelector<HTMLInputElement>('[data-avatar-letter]')!;
    expect(input.placeholder).toBe('W');
    fireEvent.input(input, { target: { value: 'ABC' } });
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(editor().querySelector('[data-avatar-save]')).toBeDisabled();
    fireEvent.input(input, { target: { value: 'Ab' } });
    expect(editor().querySelector('[data-avatar-preview] .person-avatar')).toHaveTextContent('Ab');
    fireEvent.click(editor().querySelector('[data-avatar-save]')!);
    await flush();
    expect(onSave).toHaveBeenCalledWith({ kind: 'letter', text: 'Ab' });
  });

  it('opens on the saved avatar and Auto is the default foreground with an AA note', () => {
    const { trigger } = mount({ kind: 'icon', icon: 'house', bg: '#fde68a' });
    fireEvent.click(trigger);
    expect(editor().querySelector('[data-avatar-mode="icon"]')).toHaveAttribute('aria-pressed', 'true');
    expect(editor().querySelector('[data-avatar-choice="house"]')).toHaveAttribute('aria-checked', 'true');
    const note = editor().querySelector('[data-avatar-contrast]')!;
    expect(note).toHaveAttribute('data-avatar-contrast', 'ok');
    expect(note.textContent).toMatch(/^linked-accounts-avatar-auto-note:/);
    expect((editor().querySelector('[data-avatar-preview] .person-avatar') as HTMLElement).style.color).toBe('rgb(0, 0, 0)');
  });

  it('warns when a manual foreground fails AA and offers Auto', async () => {
    const { trigger, onSave } = mount({ kind: 'letter', text: 'W', fg: '#777777', bg: '#808080' });
    fireEvent.click(trigger);
    const warn = editor().querySelector('[data-avatar-contrast="low"]')!;
    expect(warn).toHaveAttribute('role', 'status');
    expect(warn.textContent).toContain('linked-accounts-avatar-contrast-low');
    fireEvent.click(editor().querySelector('[data-avatar-use-auto]')!);
    expect(editor().querySelector('[data-avatar-contrast]')).toHaveAttribute('data-avatar-contrast', 'ok');
    fireEvent.click(editor().querySelector('[data-avatar-save]')!);
    await flush();
    expect(onSave).toHaveBeenCalledWith({ kind: 'letter', text: 'W', bg: '#808080' });
  });

  it('Escape cancels without saving; Reset goes back to the generated avatar', async () => {
    const { trigger, onSave } = mount({ kind: 'icon', icon: 'lock' });
    fireEvent.click(trigger);
    fireEvent.keyDown(editor().querySelector('[data-avatar-mode="letter"]')!, { key: 'Escape' });
    expect(editor()).toHaveAttribute('hidden');
    expect(onSave).not.toHaveBeenCalled();
    expect(document.activeElement).toBe(trigger);
    fireEvent.click(trigger);
    fireEvent.click(editor().querySelector('[data-avatar-reset]')!);
    await flush();
    expect(onSave).toHaveBeenCalledWith(undefined);
  });

  it('a refused save keeps the editor open with an error', async () => {
    const { trigger } = mount(undefined, vi.fn(async () => false));
    fireEvent.click(trigger);
    fireEvent.click(editor().querySelector('[data-avatar-mode="icon"]')!);
    fireEvent.click(editor().querySelector('[data-avatar-save]')!);
    await flush();
    expect(editor()).not.toHaveAttribute('hidden');
    expect(editor().querySelector('[role="alert"]')).toHaveTextContent('linked-accounts-avatar-failed');
  });

  it('the Settings table gives every mailbox, the sign-in one too, its avatar editor', async () => {
    const value: LinkedMailboxList = {
      primary: { accountId: '1', address: 'primary@example.org', storageAuthority: 'a' },
      accounts: [{ id: 'child', accountId: '1', address: 'child@example.org', displayName: 'Child', storageAuthority: 'b', status: 'ready', revision: 'v1' }],
      policy: { enabled: true, maxAccounts: 4, supportedAuthentication: ['password'] },
      layout: { revision: 'r', order: ['main', 'child'], nicknames: {}, avatars: { child: { kind: 'icon', icon: 'briefcase', bg: '#1d4ed8' } } },
    };
    const onAvatar = vi.fn(async () => true);
    render(() => <LinkedAccountList value={value} busy={false} onAdd={vi.fn()} onUnlink={vi.fn()} onSetFirst={vi.fn()} onMove={vi.fn()} onRename={vi.fn(async () => true)} onAvatar={onAvatar} />);
    const avatars = [...document.querySelectorAll<HTMLElement>('[data-linked-row] [data-linked-avatar]')];
    expect(avatars).toHaveLength(2);
    expect(avatars[1]!.querySelector('.person-avatar')).toHaveAttribute('data-avatar-icon', 'briefcase');
    fireEvent.click(avatars[0]!);
    const main = document.querySelector<HTMLElement>('[data-avatar-editor="main"]')!;
    fireEvent.click(main.querySelector('[data-avatar-mode="icon"]')!);
    fireEvent.click(main.querySelector('[data-avatar-choice="house"]')!);
    fireEvent.click(main.querySelector('[data-avatar-save]')!);
    await flush();
    expect(onAvatar).toHaveBeenCalledWith(null, { kind: 'icon', icon: 'house' });
  });
});
