import type {RibbonReaderActions,RibbonPreview} from './CommandRibbonContext.tsx';
import {RibbonSlot,type RibbonSlotTarget} from './compose/ribbonParts.tsx';
import {READER_ACTIONS,readerActionPlacement} from './readerActionPlacement.ts';
import type {Email} from '../api/jmap-types.ts';
import {createMessageContextMenu} from './MessageContextMenu.tsx';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import { Icon } from './Icon.tsx';
// Optional Outlook-style ribbon (plan §3 e4, §0.9). A LAYOUT PRESET beside the
// default command-palette/pane layout — OFF unless the `layout` setting is
// `'ribbon'` (see the theme slice + Settings). Tabs: Home / View / Folder.
//
// t38: the ribbon is a `CollapsibleBar` (`ui.bar.mailRibbon`). Pinned, the
// tab names and the groups are always shown; auto-hide keeps only the tab
// names, and the groups slide out over the content on hover, a click on a tab
// or keyboard focus (Esc closes them and returns focus to the tab). Every
// group is a `RibbonGroup`: exactly three rows of controls, never more, never
// scrolling vertically; when a tab's groups are too wide, the lowest-priority
// ones fold into dropdown buttons (user decision D1).
//
// a11y (t8-e1): the tab strip is a WAI-ARIA `tablist` with roving tabindex —
// exactly one tab is Tab-reachable, Arrow/Home/End move between tabs with
// selection following focus, and each tab `aria-controls` its `tabpanel`. The
// panel groups are labelled groups. Styling is token-native
// (styles/ribbon.css.ts); focus rings come from the shared a11y contract.

import { createEffect, createSignal, For, Index, onMount, onCleanup, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { loadCatalog, t } from '../i18n/index.ts';
import { THEME_OPTIONS } from '../theme/tokens.ts';
import type { Density } from '../theme/contract.css.ts';
import { settings } from '../api/settings.ts';
import { migrateLegacyRibbonCollapsed } from '../state/legacyRibbonCollapsed.ts';
import { CollapsibleBar, RibbonGroup, RibbonItem, RibbonPanel, useCollapsibleBar, type BarModeHandle } from './bars/index.ts';
import * as css from '../styles/ribbon.css.ts';
import './compose/compose-extras.css';
import './compose/richtext.css';
import * as a11y from './mailA11y.css.ts';

type RibbonTab = 'home' | 'view' | 'folder' | 'search' | 'file';

const TABS: readonly RibbonTab[] = ['home', 'view', 'folder', 'search'];
const DENSITIES: readonly Density[] = ['compact', 'cozy', 'relaxed'];

export interface RibbonProps {
  /** Open the composer (owned by the mailbox screen). */
  onCompose: () => void;
  onCloseMessage?: (()=>void)|undefined;
  /** Open the Settings panel. */
  onOpenSettings: () => void;
  /** The pin/auto-hide handle; defaults to `useBarMode('mailRibbon')` (tests pass one). */
  bar?: BarModeHandle;
  readerActions?: RibbonReaderActions | undefined;
  preview?: RibbonPreview | undefined;
  onPreviewTarget?: (target:RibbonSlotTarget|undefined)=>void;
  workspaceActions?: JSX.Element;
  layoutActions?: JSX.Element;
  searchActions?: JSX.Element;
  search?: JSX.Element;
  email?: Email | undefined;
  onOpenMessage?: (id:string)=>void;
  onOpenMessageTab?: (id:string)=>void;
  onCreateFolder?: (()=>void)|undefined;

}

export function Ribbon(props: RibbonProps): JSX.Element {
  const app = useApp();
  void loadCatalog('file-previews');void loadCatalog('mail');void loadCatalog('properties');
  const commands=createMessageContextMenu(()=>props.email,undefined,undefined,undefined,{allowRetainedMessage:true,actionFilter:row=>ribbonPlacement(row.id)!=='hidden',onOpenMessage:id=>props.onOpenMessage?.(id),onOpenMessageTab:id=>props.onOpenMessageTab?.(id)});
  const scope=()=>`${app.accountId()}:${preferenceAccount()?.generation}:${props.email?.id}:${[...app.selectedIds()].sort().join(',')}`;
  const groups=[
    {label:'mail-context-reply',icon:'reply' as const,ids:['reply','reply-all','forward','quick-reply']},
    {label:'mail-reader-move-to',icon:'moveTo' as const,ids:['archive','delete','trash','spam','junk','move','move-to','copy','snooze']},
    {label:'mail-mark-as',icon:'flag' as const,ids:['mark-as','read','flag','followup','labels','pin','create-task','quick-print','finish-followup']},
  ];
  const rows=(ids:readonly string[])=>(props.readerActions?.toolbar??commands.actions()).filter(row=>ids.includes(row.id)&&ribbonPlacement(row.id)==='toolbar').map(row=>{
    const chosen=app.selectedIds(),message=props.email;
    if(!message||!chosen.has(message.id)||chosen.size<2)return row;
    const targets=[...chosen];const runs:Record<string,()=>Promise<void>>={archive:()=>app.archiveMessage(targets),trash:()=>app.trashMessage(targets),junk:()=>app.markSpam(targets)};
    return runs[row.id]?{...row,run:runs[row.id]!}:row;
  });

  // The theme names in the View tab come from the appearance catalog.
  onMount(() => void loadCatalog('appearance'));
  // The pre-t38 device-local collapse becomes `ui.bar.mailRibbon = auto`, once.
  createEffect(() => {
    settings();
    void migrateLegacyRibbonCollapsed();
  });
  const [active, setActive] = createSignal<RibbonTab>('home');
  const tabs=()=>props.preview?[...TABS,'file' as const]:TABS;
  let previewToken:symbol|undefined;
  createEffect(()=>{const token=props.preview?.token;if(token&&token!==previewToken)setActive('file');else if(!token&&active()==='file')setActive('home');previewToken=token;});
  const readerMore=createContextMenu({label:()=>t('mail-more-actions'),scope,actions:()=>props.readerActions?.menu??[]});

  const btn = `btn btn--ghost ${css.btn} ${a11y.focusable}`;
  const large = `btn btn--ghost compose-rib__large ${css.btn} ${css.btnLarge} ${a11y.focusable}`;

  return (
    <CollapsibleBar
      id="mailRibbon"
      revealPolicy="push"
      {...(props.bar ? { bar: props.bar } : {})}
      class={`compose-ribbon workspace-command-ribbon ${css.ribbon}`}
      panelClass={css.panel}
      header={<div class="compose-ribbon__heading"><RibbonTabs tabs={tabs()} active={active()} onSelect={setActive}/><Show when={active()==='file'&&props.preview}>{preview=><strong class={css.previewTitle} title={preview().name}>{preview().name}</strong>}</Show></div>}
    >
      <div id="ribbon-panel" role="tabpanel" aria-labelledby={`ribbon-tab-${active()}`} class={css.body}>
        <Show when={active() === 'home'}>
          <RibbonPanel label={t('mail-ribbon-tab-home')}>
            <RibbonGroup label={t('mail-ribbon-group-new')} priority={30} icon="mail" testId="ribbon-group-new">
              <RibbonItem size="large">
                <button type="button" class={large} onClick={() => props.onCompose()}>
                  <Icon name="mail" />
                  <span class={css.label}>{t('mail-ribbon-compose')}</span>
                </button>
              </RibbonItem>
            </RibbonGroup>
            <Show when={props.email||props.readerActions}><For each={groups}>{group=><Show when={rows(group.ids).length}><RibbonGroup label={t(group.label)} icon={group.icon} priority={20} testId={`ribbon-message-${group.icon}`}><Index each={rows(group.ids)}>{row=><RibbonItem><RibbonCommand action={row()} scope={scope()}/></RibbonItem>}</Index></RibbonGroup></Show>}</For>
              <Show when={props.readerActions}><RibbonGroup label={t('mail-reader-actions')} icon="preferences" priority={15}><Index each={props.readerActions!.toolbar.filter(action=>!groups.some(group=>group.ids.includes(action.id)))}>{action=><RibbonItem><RibbonCommand action={action()} scope={scope()}/></RibbonItem>}</Index></RibbonGroup></Show>
              <RibbonGroup label={t('mail-more-actions')} icon="more" priority={5} testId="ribbon-message-more"><RibbonItem><button type="button" class={btn} aria-label={t('mail-more-actions')} aria-haspopup="menu" aria-expanded={props.readerActions?readerMore.isOpen():commands.isOpen()} onClick={event=>(props.readerActions?readerMore:commands).toggleBelow(event.currentTarget)}><Icon name="more"/><span class={css.label}>{t('mail-more-actions')}</span></button></RibbonItem></RibbonGroup>
            </Show>
            <Show when={props.onCloseMessage}><RibbonGroup label={t('common-close')} icon="close" priority={30}><RibbonItem><button type="button" class={btn} data-testid="workspace-close-message" onClick={()=>props.onCloseMessage?.()}><Icon name="close"/><span>{t('mail-reader-close-message')}</span></button></RibbonItem></RibbonGroup></Show>
            {props.workspaceActions}
            <RibbonGroup label={t('mail-ribbon-group-session')} priority={10} icon="signOut" testId="ribbon-group-session">
              <RibbonItem>
                <button type="button" class={btn} onClick={() => void app.logout()}>
                  <Icon name="signOut" />
                  <span class={css.label}>{t('mail-ribbon-logout')}</span>
                </button>
              </RibbonItem>
            </RibbonGroup>
          </RibbonPanel>
        </Show>

        <Show when={active() === 'view'}>
          <RibbonPanel label={t('mail-workspace-layout')}>
            {props.layoutActions}
            <RibbonGroup label={t('mail-ribbon-group-theme')} priority={30} icon="branding" testId="ribbon-group-theme">
              <RibbonItem inlineSize="11rem">
                <select
                  class={`${css.btn} ${css.select} ${a11y.focusable}`}
                  aria-label={t('mail-ribbon-group-theme')}
                  value={app.theme()}
                  onChange={(e) => app.setTheme(e.currentTarget.value as (typeof THEME_OPTIONS)[number]['value'])}
                >
                  <For each={THEME_OPTIONS}>
                    {(o) => <option value={o.value}>{t(o.labelId)}</option>}
                  </For>
                </select>
              </RibbonItem>
            </RibbonGroup>
            <RibbonGroup label={t('mail-ribbon-group-density')} priority={20} icon="preferences" testId="ribbon-group-density">
              <For each={DENSITIES}>
                {(d) => (
                  <RibbonItem>
                    <button type="button" class={btn} aria-pressed={app.density() === d} onClick={() => app.setDensity(d)}>
                      <span class={css.label}>{t(`mail-density-${d}`)}</span>
                    </button>
                  </RibbonItem>
                )}
              </For>
            </RibbonGroup>
            <RibbonGroup label={t('mail-ribbon-group-settings')} priority={10} icon="settings" testId="ribbon-group-settings">
              <RibbonItem>
                <button type="button" class={btn} onClick={() => props.onOpenSettings()}>
                  <Icon name="settings" />
                  <span class={css.label}>{t('mail-nav-settings')}</span>
                </button>
              </RibbonItem>
            </RibbonGroup>
          </RibbonPanel>
        </Show>

        <Show when={active() === 'folder'}>
          <RibbonPanel label={t('mail-ribbon-tab-folder')}>
            <RibbonGroup label={t('mail-ribbon-group-folders')} priority={10} icon="folder" testId="ribbon-group-folders">
              <Show when={props.onCreateFolder}><RibbonItem><button type="button" class={btn} onClick={()=>props.onCreateFolder?.()}><Icon name="folderPlus"/><span class={css.label}>{t('folder-create')}</span></button></RibbonItem></Show>
              <For each={app.mailboxes()}>
                {(box) => (
                  <RibbonItem>
                    <button
                      type="button"
                      class={btn}
                      aria-pressed={app.selectedMailboxId() === box.id}
                      data-tooltip={box.name}
                      onClick={() => void app.selectMailbox(box.id)}
                    >
                      <span class={css.label}>{box.name}</span>
                    </button>
                  </RibbonItem>
                )}
              </For>
            </RibbonGroup>
          </RibbonPanel>
        </Show>
        <div hidden={active() !== 'search'} inert={active() !== 'search'}>
          <div class={css.search} data-ribbon-search="">{props.search}<div class={css.commands} role="toolbar" aria-label={t('mail-order-by')}>{props.searchActions}</div></div>
        </div>
        <Show when={active()==='file'&&props.preview}>
          <RibbonPanel label={t('preview-tools-title')}><RibbonSlot name="file-preview" onTarget={target=>props.onPreviewTarget?.(target)}/></RibbonPanel>
        </Show>
      </div>
      {commands.menu()}
      {readerMore.menu()}
    </CollapsibleBar>
  );
}

/** The always-visible tab names; in auto-hide mode a click (or focus) opens the bar. */
function RibbonTabs(props: { tabs:readonly RibbonTab[]; active: RibbonTab; onSelect: (tab: RibbonTab) => void }): JSX.Element {
  const bar = useCollapsibleBar();
  const tabEls: (HTMLButtonElement | undefined)[] = [];

  /** Roving-tabindex keyboard handler: Arrow/Home/End move + activate the tab. */
  function onTabKeyDown(e: KeyboardEvent): void {
    const idx = props.tabs.indexOf(props.active);
    let next = idx;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (idx + 1) % props.tabs.length;
    else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (idx - 1 + props.tabs.length) % props.tabs.length;
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = props.tabs.length - 1;
    else return;
    e.preventDefault();
    props.onSelect(props.tabs[next]!);
    tabEls[next]?.focus();
  }

  return (
    <div class={css.tabs}>
      <div class={css.tablist} role="tablist" aria-label={t('mail-ribbon-label')} onKeyDown={onTabKeyDown}>
        <For each={props.tabs}>
          {(id, i) => (
            <button
              type="button"
              ref={(el) => (tabEls[i()] = el)}
              role="tab"
              id={`ribbon-tab-${id}`}
              aria-controls="ribbon-panel"
              aria-selected={props.active === id}
              aria-expanded={bar?.mode() === 'auto' ? (bar.expanded() ? 'true' : 'false') : undefined}
              tabindex={props.active === id ? 0 : -1}
              class={`${props.active === id ? `${css.tab} ${css.tabActive}` : css.tab} ${a11y.focusable}`}
              onClick={(e) => {
                props.onSelect(id);
                bar?.open(e.currentTarget);
              }}
            >
              {t(id==='file'?'preview-title':id==='search'?'mail-search':id==='view'?'mail-workspace-layout':`mail-ribbon-tab-${id}`)}
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

function RibbonCommand(props:{action:ContextAction;scope:string}):JSX.Element{
 const app=useApp();
 const bar=useCollapsibleBar();
 const menu=createContextMenu({label:()=>props.action.label,scope:()=>props.scope,actions:()=>props.action.children??[],searchLabel:()=>props.action.searchLabel??'',onError:()=>app.showToast('error',t('mail-context-failed'))});
 createEffect(()=>{if(menu.isOpen()&&bar){const release=bar.hold();onCleanup(release);}});
 return <><button type="button" class={`btn btn--ghost ${css.btn} ${a11y.focusable}`} disabled={props.action.disabled} aria-pressed={props.action.checked} title={props.action.title??props.action.label} aria-label={props.action.label} aria-haspopup={props.action.children?'menu':undefined} aria-expanded={props.action.children?menu.isOpen():undefined} onClick={event=>{if(props.action.children)menu.toggleBelow(event.currentTarget);else void Promise.resolve().then(()=>props.action.run()).catch(()=>app.showToast('error',t('mail-context-failed')));}}><Icon name={props.action.icon}/><span class={css.label}>{props.action.label}</span><Show when={props.action.children}><Icon name="chevronDown"/></Show></button>{menu.menu()}</>;
}

function ribbonPlacement(id:string){
 const aliases:Record<string,string>={trash:'delete',junk:'spam','move-to':'move','quick-print':'print',followup:'flag',download:'export'};
 const key=aliases[id]??id;
 return READER_ACTIONS.some(action=>action.id===key)?readerActionPlacement(key):'toolbar';
}
