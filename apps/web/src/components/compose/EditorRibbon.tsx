// The rich-text editor's ribbon (t49), shared by the composer and the
// signature editor so both get the same bar.
//
// It is one `CollapsibleBar` (the `composerRibbon` bar: the same pin or
// auto-hide preference everywhere the editor is). The undo/redo history and the
// tab names sit in the header. Below them is one `RibbonPanel` per tab. Each
// editor tab (Format, Insert, Table) has a `RibbonSlot`, and the
// `RichTextEditor` renders its own groups there (`ribbonSlots`). A host adds
// its own groups before or after that slot (the composer adds Recipients,
// Attachments, Signature and Assist) and can add tabs of its own (Security,
// Delivery). Hidden tabs stay mounted, so a host's live state in them (the
// composer's security evaluation) keeps running.
//
// `BodyModeGroup` is the one select that switches the body between plain text,
// the rich editor and HTML source. Both hosts use it.

import { createSignal, For, Show, type JSX } from 'solid-js';
import {
  CollapsibleBar,
  RibbonGroup,
  RibbonItem,
  RibbonPanel,
  useCollapsibleBar,
  type BarModeHandle,
  type CollapsibleBarApi,
} from '../bars/index.ts';
import { ComposerHistory } from './ComposerHistory.tsx';
import { RibbonSlot, type RibbonSlotTarget } from './ribbonParts.tsx';
import type { RichTextApi } from './RichTextEditor.tsx';
import { t } from '../../i18n/index.ts';

export type EditorTab = 'format' | 'insert' | 'table';
export const EDITOR_TABS: readonly EditorTab[] = ['format', 'insert', 'table'];
export type EditorSlots = Partial<Record<EditorTab, RibbonSlotTarget>>;

const isEditorTab = (tab: string): tab is EditorTab => (EDITOR_TABS as readonly string[]).includes(tab);

/** The ribbon's state, created by its host: the open tab, the editor's slots and the bar. */
export interface EditorRibbonState<T extends string> {
  tab: () => EditorTab | T;
  setTab: (tab: EditorTab | T) => void;
  /** Switch to `tab` and, in auto-hide mode, reveal the ribbon (something there needs the user). */
  show: (tab: EditorTab | T) => void;
  /** The open tab when it is one of the editor's, else `null` (for `RichTextEditor.ribbonTab`). */
  editorTab: () => EditorTab | null;
  /** Where the editor renders each tab's groups (for `RichTextEditor.ribbonSlots`). */
  slots: () => EditorSlots;
  setSlot: (tab: EditorTab) => (target: RibbonSlotTarget | undefined) => void;
  attachBar: (bar: CollapsibleBarApi | undefined) => void;
}

export function createEditorRibbon<T extends string = never>(initial: EditorTab | T = 'format'): EditorRibbonState<T> {
  const [tab, setTabSignal] = createSignal<EditorTab | T>(initial);
  const [slots, setSlots] = createSignal<EditorSlots>({});
  let bar: CollapsibleBarApi | undefined;
  const setTab = (next: EditorTab | T): void => {
    setTabSignal(() => next);
  };
  return {
    tab,
    setTab,
    show(next) {
      setTab(next);
      bar?.open();
    },
    editorTab: () => {
      const current = tab();
      return isEditorTab(current) ? current : null;
    },
    slots,
    setSlot: (name) => (target) => {
      setSlots((cur) => {
        const next = { ...cur };
        if (target) next[name] = target;
        else delete next[name];
        return next;
      });
    },
    attachBar(api) {
      bar = api;
    },
  };
}

/** One tab of the ribbon: its panel's groups around the editor's slot (editor tabs only). */
export interface EditorRibbonTab<T extends string> {
  id: EditorTab | T;
  /** The tab's name; defaults to the editor tab's name. */
  label?: string;
  before?: () => JSX.Element;
  after?: () => JSX.Element;
}

export interface EditorRibbonProps<T extends string> {
  state: EditorRibbonState<T>;
  /** Every panel, in order. All stay mounted; only `visible` ones get a tab. */
  tabs: readonly EditorRibbonTab<T>[];
  visible: () => readonly (EditorTab | T)[];
  /** The editor whose history the header's undo and redo act on. */
  history: () => RichTextApi | null;
  showHistory: () => boolean;
  /** The bar's mode handle; defaults to the `composerRibbon` preference. */
  bar?: BarModeHandle;
  class?: string;
}

const tabLabel = <T extends string>(spec: EditorRibbonTab<T>): string => spec.label ?? t(`mail-compose-ribbon-${spec.id}`);

export function EditorRibbon<T extends string>(props: EditorRibbonProps<T>): JSX.Element {
  const state = props.state;
  const Capture = (): JSX.Element => {
    state.attachBar(useCollapsibleBar());
    return null;
  };
  const Heading = (): JSX.Element => {
    const bar = useCollapsibleBar();
    const label = (id: EditorTab | T): string => {
      const spec = props.tabs.find((tab) => tab.id === id);
      return spec ? tabLabel(spec) : id;
    };
    return (
      <div class="compose-ribbon__heading" data-detached-local>
        <Show when={props.showHistory()}>
          <ComposerHistory api={props.history()} />
        </Show>
        <div class="compose-rt__tabs" role="tablist" aria-label={t('mail-compose-ribbon')}>
          <For each={props.visible()}>
            {(tab) => (
              <button
                type="button"
                role="tab"
                aria-selected={state.tab() === tab}
                aria-expanded={bar?.mode() === 'auto' ? (bar.expanded() && state.tab() === tab ? 'true' : 'false') : undefined}
                aria-controls={bar?.panelId}
                onClick={(event) => {
                  // Auto-hide: a tab opens the ribbon on itself; the open tab, clicked again, hides it (Office).
                  if (bar?.mode() === 'auto' && bar.expanded() && state.tab() === tab) {
                    bar.close();
                    return;
                  }
                  state.setTab(tab);
                  bar?.open(event.currentTarget);
                }}
              >
                {label(tab)}
              </button>
            )}
          </For>
        </div>
      </div>
    );
  };
  return (
    <CollapsibleBar
      id="composerRibbon"
      revealPolicy="push"
      {...(props.bar ? { bar: props.bar } : {})}
      class={props.class ? `compose-ribbon ${props.class}` : 'compose-ribbon'}
      label={t('mail-compose-ribbon')}
      header={<Heading />}
    >
      <Capture />
      <div class="compose-ribbon__content">
        <For each={props.tabs}>
          {(spec) => (
            <div class="compose-ribbon__panel" data-ribbon-tab={spec.id} hidden={state.tab() !== spec.id} inert={state.tab() !== spec.id}>
              <RibbonPanel label={tabLabel(spec)}>
                {spec.before?.()}
                <Show when={isEditorTab(spec.id) ? spec.id : undefined}>
                  {(name) => <RibbonSlot name={name()} onTarget={state.setSlot(name())} />}
                </Show>
                {spec.after?.()}
              </RibbonPanel>
            </div>
          )}
        </For>
      </div>
    </CollapsibleBar>
  );
}

export type BodyMode = 'plain' | 'rich' | 'html';

/** The body's mode: plain text, the rich editor, or its HTML source. One narrow
 *  select, kept above Paragraph in folding order, since it is needed at any width. */
export function BodyModeGroup(props: {
  value: () => BodyMode;
  onChange: (mode: BodyMode) => void;
  label: string;
  testId?: string;
  selectTestId?: string;
}): JSX.Element {
  return (
    <RibbonGroup label={props.label} priority={85} icon="mail" testId={props.testId ?? 'compose-group-message-type'}>
      <RibbonItem>
        <select
          data-testid={props.selectTestId ?? 'format-toggle'}
          aria-label={props.label}
          value={props.value()}
          onChange={(event) => props.onChange(event.currentTarget.value as BodyMode)}
        >
          <option value="plain">{t('mail-compose-format-plain')}</option>
          <option value="rich">{t('mail-compose-format-rich')}</option>
          <option value="html">{t('mail-compose-html-source')}</option>
        </select>
      </RibbonItem>
    </RibbonGroup>
  );
}
