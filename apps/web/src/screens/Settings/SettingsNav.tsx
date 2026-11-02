// t66: the Settings section navigation, one labelled group per category. The
// headings are labels, not entries: arrow keys move between section buttons
// only, and each group is announced by its heading (`role="group"` +
// `aria-labelledby`).
import { createUniqueId, For, Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { Icon } from '../../components/Icon.tsx';
import {createContextMenu} from '../../components/ContextMenu.tsx';
import { SETTINGS_CATEGORIES, visibleCategories } from './settingsCategories.ts';
import * as workspace from './workspace.css.ts';

export interface SettingsNavProps {
  current: () => string;
  query: () => string;
  onQueryChange?: (query: string) => void;
  supported: (section: string) => boolean;
  onSelect: (section: string) => void;
}

const NEXT = new Set(['ArrowDown', 'ArrowRight']);
const PREVIOUS = new Set(['ArrowUp', 'ArrowLeft']);

export function SettingsNav(props: SettingsNavProps): JSX.Element {
  let nav!: HTMLElement;
  const groups = () => visibleCategories(props.supported, props.query(), id => t(id));
  const current=()=>SETTINGS_CATEGORIES.flatMap(category=>category.sections).find(entry=>entry.id===props.current());
  const labelId=`settings-current-${createUniqueId()}`;
  const menu=createContextMenu({label:()=>t('settings-sections'),scope:()=>`${props.current()}:${props.query()}:${groups().flatMap(group=>group.sections.map(entry=>entry.id)).join(',')}`,focusCurrent:true,actions:()=>groups().flatMap((group,groupIndex)=>group.sections.map((entry,index)=>({id:entry.id,label:t(entry.label),icon:entry.icon,radio:true,checked:props.current()===entry.id,current:props.current()===entry.id,separatorBefore:groupIndex>0&&index===0,run:()=>props.onSelect(entry.id)})))});
  function openPicker(trigger:HTMLButtonElement):void {
    menu.openBelow(trigger);
    queueMicrotask(()=>{
      // Keep the chosen section visible even when it is near the end of a
      // viewport-sized menu. Focus alone deliberately avoids page scrolling.
      const panel=Array.from(trigger.ownerDocument.querySelectorAll<HTMLElement>('[role="menu"]')).find(element=>element.getAttribute('aria-label')===t('settings-sections'));
      const selected=panel?.querySelector<HTMLElement>('[aria-checked="true"]');
      if(panel&&selected){const item=selected.getBoundingClientRect(),box=panel.getBoundingClientRect();if(item.bottom>box.bottom)panel.scrollTop+=item.bottom-box.bottom;else if(item.top<box.top)panel.scrollTop-=box.top-item.top;}
    });
  }
  const onKeyDown = (event: KeyboardEvent): void => {
    const buttons = [...nav.querySelectorAll<HTMLButtonElement>('button[data-settings-entry]')];
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
    <nav ref={nav} class={workspace.navRoot} aria-label={t('settings-sections')} onKeyDown={onKeyDown}>
      <div class={workspace.navHeader}><span class={workspace.navTitle}>{t('settings-title')}</span><input class={workspace.search} type="search" aria-label={t('settings-search-sections')} placeholder={t('settings-search-sections')} value={props.query()} onInput={event=>props.onQueryChange?.(event.currentTarget.value)}/></div>
      <button type="button" class={`btn btn--ghost ${workspace.sectionPicker}`} data-settings-picker aria-label={t('settings-sections')} aria-describedby={labelId} title={current()?t(current()!.label):t('settings-sections')} aria-haspopup="menu" aria-expanded={menu.isOpen()} disabled={!groups().length} onClick={event=>{if(menu.isOpen())menu.close();else openPicker(event.currentTarget);}} onKeyDown={event=>{if(event.key==='ArrowDown'||event.key==='ArrowUp'){event.preventDefault();openPicker(event.currentTarget);}}}><Icon name={current()?.icon??'preferences'}/><span id={labelId}>{current()?t(current()!.label):t('settings-sections')}</span><Icon name="chevronDown"/></button>
      <Show when={groups().length} fallback={<p role="status" class={workspace.navEmpty}>{t('settings-search-no-sections')}</p>}><div class={workspace.nav}>
      <For each={groups()}>{group => {
        const headingId = `settings-category-${group.category.id}`;
        return (
          <div class={workspace.group} role="group" aria-labelledby={headingId} data-settings-category={group.category.id}>
            <span class={workspace.groupHeading} id={headingId}>{t(group.category.label)}</span>
            <For each={group.sections}>{entry => (
              <button type="button" data-settings-entry={entry.id} aria-current={props.current() === entry.id ? 'page' : undefined} onClick={() => props.onSelect(entry.id)}>
                <Icon name={entry.icon} /><span>{t(entry.label)}</span>
              </button>
            )}</For>
          </div>
        );
      }}</For>
      </div></Show>
      {menu.menu()}
    </nav>
  );
}

export default SettingsNav;
