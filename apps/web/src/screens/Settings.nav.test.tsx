// t66: the Settings navigation is grouped into categories.
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, screen, within } from '@solidjs/testing-library';
import { Settings } from './Settings.tsx';
import { AppContext } from '../state/context.ts';
import { createThemeSlice } from '../state/slices/theme.ts';
import type { SliceContext } from '../state/slices/context.ts';
import type { AppState } from '../state/store.ts';
import { clearSettings } from '../api/settings.ts';
import { createSettingsNavigation } from './Settings/navigation.tsx';
import { SETTINGS_CATEGORIES, categoryOf, visibleCategories } from './Settings/settingsCategories.ts';

function renderEmbedded(linked = false, section = 'appearance') {
  const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;
  const app = { ...createThemeSlice(ctx), accountId: () => null, linkedMailboxId: () => (linked ? 'linked-1' : null) } as unknown as AppState;
  const navigation = createSettingsNavigation({ section });
  render(() => (
    <AppContext.Provider value={app}>
      <Settings embedded navigation={navigation} onClose={vi.fn()} />
    </AppContext.Provider>
  ));
  return { navigation, nav: screen.getByRole('navigation', { name: 'Settings sections' }) };
}

const groupNames = (nav: HTMLElement): string[] => within(nav).getAllByRole('group').map(group => group.getAttribute('aria-labelledby') ?? '').map(id => document.getElementById(id)?.textContent ?? '');
const desktopEntries=(nav:HTMLElement):HTMLButtonElement[]=>Array.from(nav.querySelectorAll<HTMLButtonElement>('button[data-settings-entry]'));

describe('Settings navigation categories (t66)', () => {
  beforeEach(() => { clearSettings(); localStorage.clear(); });

  it('every section belongs to exactly one category', () => {
    const ids = SETTINGS_CATEGORIES.flatMap(category => category.sections.map(entry => entry.id));
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toHaveLength(22);
    expect(categoryOf('linked-accounts')?.id).toBe('account');
    expect(categoryOf('metadata')?.id).toBe('about');
    expect(categoryOf('preferences')).toBeUndefined();
  });

  it('groups the sections under labelled category headings, in order', () => {
    const { nav } = renderEmbedded();
    expect(groupNames(nav)).toEqual(['Account', 'Mail', 'Interface', 'Integrations', 'About']);
    const account = within(nav).getByRole('group', { name: 'Account' });
    expect(within(account).getAllByRole('button').map(button => button.textContent)).toEqual(['Linked mailboxes', 'Identities', 'Signatures', 'Security centre', 'Privacy']);
    // Headings are labels, not entries.
    expect(within(nav).queryByRole('button', { name: 'Account' })).toBeNull();
    expect(desktopEntries(nav)).toHaveLength(22);
  });

  it('hides a category whose sections are all unavailable', () => {
    const { nav } = renderEmbedded(true);
    expect(groupNames(nav)).toEqual(['Account', 'Mail', 'Interface']);
    const account = within(nav).getByRole('group', { name: 'Account' });
    expect(within(account).getAllByRole('button').map(button => button.textContent)).toEqual(['Linked mailboxes', 'Identities', 'Signatures']);
  });

  it('search keeps the category heading as context and drops empty categories', () => {
    const { nav } = renderEmbedded();
    fireEvent.input(within(nav).getByRole('searchbox', { name: 'Search settings' }), { target: { value: 'nextcloud' } });
    expect(groupNames(nav)).toEqual(['Integrations']);
    expect(within(within(nav).getByRole('group', { name: 'Integrations' })).getByRole('button', { name: 'Nextcloud accounts' })).toBeInTheDocument();
    // A category name is a search term too.
    fireEvent.input(within(nav).getByRole('searchbox', { name: 'Search settings' }), { target: { value: 'integrations' } });
    expect(desktopEntries(nav).map(button => button.textContent)).toEqual(['Nextcloud accounts', 'Package tracking', 'API access']);
    fireEvent.input(within(nav).getByRole('searchbox', { name: 'Search settings' }), { target: { value: 'zzzz' } });
    expect(within(nav).queryAllByRole('group')).toHaveLength(0);
    expect(within(nav).getByRole('status')).toHaveTextContent('No matching settings sections.');
  });

  it('arrow keys move between entries across categories and skip headings', () => {
    const { nav } = renderEmbedded();
    const buttons = desktopEntries(nav);
    const privacy = within(nav).getByRole('button', { name: 'Privacy' });
    privacy.focus();
    fireEvent.keyDown(privacy, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(within(nav).getByRole('button', { name: 'Message display' }));
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowUp' });
    expect(document.activeElement).toBe(privacy);
    fireEvent.keyDown(privacy, { key: 'End' });
    expect(document.activeElement).toBe(buttons.at(-1));
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(buttons[0]);
    fireEvent.keyDown(buttons[0]!, { key: 'ArrowUp' });
    expect(document.activeElement).toBe(buttons.at(-1));
  });

  it('selecting an entry navigates and shows the category above the section', async () => {
    const { nav, navigation } = renderEmbedded();
    expect(screen.getByTestId('settings-breadcrumb')).toHaveTextContent('Interface');
    fireEvent.click(within(nav).getByRole('button', { name: 'Mail rules' }));
    await vi.waitFor(() => expect(navigation.route().section).toBe('rules'));
    expect(within(nav).getByRole('button', { name: 'Mail rules' })).toHaveAttribute('aria-current', 'page');
    expect(screen.getByTestId('settings-breadcrumb')).toHaveTextContent('Mail');
  });

  it('visibleCategories filters by support and query', () => {
    const groups = visibleCategories(section => section !== 'metadata', 'receipt', id => id);
    expect(groups.map(group => group.category.id)).toEqual(['mail']);
  });

  it('keeps the embedded Settings label and search in navigation without a separate heading or close cross',()=>{
    const {nav}=renderEmbedded();
    expect(screen.queryByRole('heading',{name:'Settings'})).toBeNull();
    expect(screen.queryByRole('button',{name:'Close settings'})).toBeNull();
    const label=within(nav).getByText('Settings',{exact:true}),search=within(nav).getByRole('searchbox',{name:'Search settings'});
    expect(label).toBeVisible();expect(label.compareDocumentPosition(search)&Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    fireEvent.input(search,{target:{value:'server metadata'}});
    expect(desktopEntries(nav).map(button=>button.textContent)).toEqual(['Server information']);
    fireEvent.click(within(nav).getByRole('button',{name:'Settings sections'}));
    expect(screen.getByRole('menuitemradio',{name:'Server information'})).toBeVisible();
  });

  it('retains the close control for modal Settings',()=>{
    const ctx={client:{},showToast:vi.fn()} as unknown as SliceContext;
    const app={...createThemeSlice(ctx),accountId:()=>null} as unknown as AppState,onClose=vi.fn();
    render(()=><AppContext.Provider value={app}><Settings onClose={onClose}/></AppContext.Provider>);
    expect(screen.getByRole('dialog',{name:'Settings'})).toBeVisible();
    fireEvent.click(screen.getByRole('button',{name:'Close settings'}));expect(onClose).toHaveBeenCalledOnce();
  });
});
