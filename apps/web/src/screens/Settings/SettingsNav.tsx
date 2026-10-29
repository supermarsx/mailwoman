// t66: the Settings section navigation, one labelled group per category. The
// headings are labels, not entries: arrow keys move between section buttons
// only, and each group is announced by its heading (`role="group"` +
// `aria-labelledby`).
import { For, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { Icon } from '../../components/Icon.tsx';
import { visibleCategories } from './settingsCategories.ts';
import * as workspace from './workspace.css.ts';

export interface SettingsNavProps {
  current: () => string;
  query: () => string;
  supported: (section: string) => boolean;
  onSelect: (section: string) => void;
}

const NEXT = new Set(['ArrowDown', 'ArrowRight']);
const PREVIOUS = new Set(['ArrowUp', 'ArrowLeft']);

export function SettingsNav(props: SettingsNavProps): JSX.Element {
  let nav!: HTMLElement;
  const groups = () => visibleCategories(props.supported, props.query(), id => t(id));
  const onKeyDown = (event: KeyboardEvent): void => {
    const buttons = [...nav.querySelectorAll<HTMLButtonElement>('button[data-settings-section]')];
    const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (at < 0 || buttons.length === 0) return;
    // In a right-to-left layout the horizontal arrows swap meaning.
    const rtl = getComputedStyle(nav).direction === 'rtl';
    const key = rtl && event.key === 'ArrowLeft' ? 'ArrowRight' : rtl && event.key === 'ArrowRight' ? 'ArrowLeft' : event.key;
    let next: number;
    if (NEXT.has(key)) next = (at + 1) % buttons.length;
    else if (PREVIOUS.has(key)) next = (at - 1 + buttons.length) % buttons.length;
    else if (key === 'Home') next = 0;
    else if (key === 'End') next = buttons.length - 1;
    else return;
    event.preventDefault();
    buttons[next]?.focus();
  };
  return (
    <nav ref={nav} class={workspace.nav} aria-label={t('settings-sections')} onKeyDown={onKeyDown}>
      <For each={groups()} fallback={<p role="status">{t('settings-search-no-sections')}</p>}>{group => {
        const headingId = `settings-category-${group.category.id}`;
        return (
          <div class={workspace.group} role="group" aria-labelledby={headingId} data-settings-category={group.category.id}>
            <span class={workspace.groupHeading} id={headingId}>{t(group.category.label)}</span>
            <For each={group.sections}>{entry => (
              <button type="button" data-settings-section={entry.id} aria-current={props.current() === entry.id ? 'page' : undefined} onClick={() => props.onSelect(entry.id)}>
                <Icon name={entry.icon} /><span>{t(entry.label)}</span>
              </button>
            )}</For>
          </div>
        );
      }}</For>
    </nav>
  );
}

export default SettingsNav;
