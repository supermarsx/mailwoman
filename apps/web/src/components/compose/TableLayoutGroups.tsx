// The Table layout tab's groups: Rows, Columns, Cells, then shading and the
// table itself. Each is a three-row ribbon group with a title below and an
// icon, which folds into a dropdown when the ribbon is narrow (RibbonGroup).
// The size controls act on the selected cells through `tableLayout.ts`.

import { For, type JSX } from 'solid-js';
import type { Command, EditorState } from 'prosemirror-state';
import { addColumnAfter, addColumnBefore, addRowAfter, addRowBefore, deleteColumn, deleteRow, deleteTable, mergeCells, splitCell, toggleHeaderRow } from 'prosemirror-tables';
import { RibbonGroup, RibbonItem } from '../bars/index.ts';
import { Icon, type IconName } from '../Icon.tsx';
import { t } from '../../i18n/index.ts';
import * as a11y from '../mailA11y.css.ts';
import { cellLook, COLUMN_WIDTHS, PADDING_PRESETS, ROW_HEIGHTS, setCellAlign, setCellPaddingPreset, setColumnWidth, setRowHeight, type CellAlign, type PaddingPreset } from './tableLayout.ts';

export interface TableLayoutProps {
  state: () => EditorState | null;
  /** Run a command on the editor, reporting a refusal. */
  execute: (command: Command) => void;
  /** The editor's shading and its "clear" buttons, as they are elsewhere. */
  shading: () => JSX.Element;
  clearShading: () => JSX.Element;
}

const ALIGNS: readonly [CellAlign, IconName][] = [['left', 'alignLeft'], ['center', 'alignCenter'], ['right', 'alignRight']];

export function TableLayoutGroups(props: TableLayoutProps): JSX.Element {
  const look = () => {
    const state = props.state();
    return state ? cellLook(state) : null;
  };
  const enabled = (command: Command): boolean => {
    const state = props.state();
    return state !== null && command(state);
  };
  const button = (name: string, command: Command, icon: IconName): JSX.Element => (
    <button
      type="button"
      class={`compose-rt__btn ${a11y.focusable}`}
      disabled={!enabled(command)}
      data-testid={`rt-table-${name}`}
      aria-label={t(`mail-editor-table-${name}`)}
      data-tooltip={t(`mail-editor-table-${name}`)}
      onMouseDown={(event) => event.preventDefault()}
      onClick={() => props.execute(command)}
    >
      <Icon name={icon} />
    </button>
  );
  const labelled = (name: string, command: Command, icon: IconName): JSX.Element => (
    <button
      type="button"
      class={`compose-rt__btn ${a11y.focusable}`}
      disabled={!enabled(command)}
      data-testid={`rt-table-${name}`}
      data-tooltip={t(`mail-editor-table-${name}`)}
      onMouseDown={(event) => event.preventDefault()}
      onClick={() => props.execute(command)}
    >
      <Icon name={icon} /><span class="compose-rib__text">{t(`mail-editor-table-${name}`)}</span>
    </button>
  );
  const row = (...controls: JSX.Element[]): JSX.Element => <div class="compose-rib__row">{controls}</div>;
  const select = (id: string, label: string, value: string, options: readonly [string, string][], onPick: (value: string) => void): JSX.Element => (
    <label class="compose-rib__field">
      <span class="compose-rib__text">{label}</span>
      <select data-testid={id} aria-label={label} disabled={look() === null} value={value} onChange={(event) => onPick(event.currentTarget.value)}>
        <For each={options}>{([v, text]) => <option value={v}>{text}</option>}</For>
      </select>
    </label>
  );
  const size = (value: number | null): string => (value === null ? '' : String(value));

  return (
    <>
      <RibbonGroup label={t('mail-editor-table-group-rows')} priority={30} icon="tableRowAfter" testId="rt-group-table-rows">
        <RibbonItem>{row(button('row-before', addRowBefore, 'tableRowBefore'), button('row-after', addRowAfter, 'tableRowAfter'), button('row-delete', deleteRow, 'tableRowDelete'))}</RibbonItem>
        <RibbonItem>{labelled('header', toggleHeaderRow, 'tableHeader')}</RibbonItem>
        <RibbonItem>{select('rt-table-row-height', t('mail-editor-table-row-height'), size(look()?.rowHeight ?? null), [['', t('mail-editor-table-size-auto')], ...ROW_HEIGHTS.map((px): [string, string] => [String(px), `${px} px`])], (v) => props.execute(setRowHeight(v === '' ? null : Number(v))))}</RibbonItem>
      </RibbonGroup>
      <RibbonGroup label={t('mail-editor-table-group-columns')} priority={29} icon="tableColumnAfter" testId="rt-group-table-columns">
        <RibbonItem>{row(button('column-before', addColumnBefore, 'tableColumnBefore'), button('column-after', addColumnAfter, 'tableColumnAfter'), button('column-delete', deleteColumn, 'tableColumnDelete'))}</RibbonItem>
        <RibbonItem>{select('rt-table-column-width', t('mail-editor-table-column-width'), size(look()?.columnWidth ?? null), [['', t('mail-editor-table-size-auto')], ...COLUMN_WIDTHS.map((px): [string, string] => [String(px), `${px} px`])], (v) => props.execute(setColumnWidth(v === '' ? null : Number(v))))}</RibbonItem>
      </RibbonGroup>
      <RibbonGroup label={t('mail-editor-table-group-cells')} priority={28} icon="tableMerge" testId="rt-group-table-cells">
        <RibbonItem>{row(button('merge', mergeCells, 'tableMerge'), button('split', splitCell, 'tableSplit'))}</RibbonItem>
        <RibbonItem>{select('rt-table-padding', t('mail-editor-table-padding'), paddingKey(look()), [['', '—'], ...(Object.keys(PADDING_PRESETS) as PaddingPreset[]).map((key): [string, string] => [key, `${t(`mail-editor-table-padding-${key}`)} (${PADDING_PRESETS[key].vertical}/${PADDING_PRESETS[key].horizontal})`])], (v) => { if (v in PADDING_PRESETS) props.execute(setCellPaddingPreset(v as PaddingPreset)); })}</RibbonItem>
        <RibbonItem>{row(<For each={ALIGNS}>{([align, icon]) => (
          <button
            type="button"
            class={`compose-rt__btn ${a11y.focusable}`}
            classList={{ 'compose-rt__btn--active': look()?.align === align }}
            aria-pressed={look()?.align === align}
            disabled={look() === null}
            data-testid={`rt-table-align-${align}`}
            aria-label={t(`mail-editor-table-align-${align}`)}
            data-tooltip={t(`mail-editor-table-align-${align}`)}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => props.execute(setCellAlign(align))}
          ><Icon name={icon} /></button>
        )}</For>)}</RibbonItem>
      </RibbonGroup>
      <RibbonGroup label={t('mail-editor-table-shading')} priority={25} icon="paint" testId="rt-group-table-shading">
        <RibbonItem>{props.shading()}</RibbonItem>
        <RibbonItem>{props.clearShading()}</RibbonItem>
      </RibbonGroup>
      <RibbonGroup label={t('mail-compose-ribbon-table')} priority={20} icon="table" testId="rt-group-table">
        <RibbonItem>{labelled('delete', deleteTable, 'delete')}</RibbonItem>
      </RibbonGroup>
    </>
  );
}

function paddingKey(look: ReturnType<typeof cellLook>): string {
  if (look === null) return '';
  for (const key of Object.keys(PADDING_PRESETS) as PaddingPreset[]) {
    if (PADDING_PRESETS[key].vertical === look.paddingVertical && PADDING_PRESETS[key].horizontal === look.paddingHorizontal) return key;
  }
  return '';
}
