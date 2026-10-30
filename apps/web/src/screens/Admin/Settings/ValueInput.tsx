// One editor for a setting value, chosen by the registry's declared kind
// (worker f01): a checkbox for on/off, a number field with its bounds, a list
// for a fixed set of words or a language, a text field with its length, and a
// JSON text area for structured values. The caller wires label, help and error
// (`FormField`) and checks the draft with `parseDraft` before any request.

import { For, Match, Switch, type JSX } from 'solid-js';
import { loadCatalog, formatLanguageName, t } from '../../../i18n';
import type { SettingKind } from '../../../api/admin/settings.ts';
import type { FieldWiring } from '../shell/FormField.tsx';
import type { Draft } from './value.ts';
import { isJsonKind } from './value.ts';
import {TemplateLibraryEditor} from '../../../components/compose/TemplateLibraryEditor.tsx';
import {templateRows} from '../../../components/compose/responseTemplateModel.ts';
import * as ux from '../shell/ux.css.ts';

export interface ValueInputProps {
  kind: SettingKind;
  draft: Draft;
  onDraft: (d: Draft) => void;
  wiring: FieldWiring;
  disabled?: boolean;
  /** The checkbox's text (the value's meaning), for `bool`. */
  checkboxLabel?: string;
  /** Allows an empty value ("Not set"). */
  nullable?: boolean;
}

export function ValueInput(props: ValueInputProps): JSX.Element {
  void loadCatalog('admin-settings'); // t112: its ids show wherever it is mounted
  const text = (): string => (typeof props.draft === 'string' ? props.draft : '');
  const options = (): readonly string[] => props.kind.values ?? [];
  const label = (v: string): string =>
    props.kind.type === 'locale' && v !== 'browser' ? `${formatLanguageName(v, v)} (${v})` : v;
  return (
    <Switch
      fallback={
        <input
          id={props.wiring.id}
          class={ux.control}
          type={
            props.kind.format === 'email'
              ? 'email'
              : props.kind.format === 'httpsUrl' || props.kind.format === 'httpUrl'
                ? 'url'
                : 'text'
          }
          maxLength={props.kind.maxLength}
          aria-describedby={props.wiring.describedBy}
          aria-invalid={props.wiring.invalid}
          disabled={props.disabled}
          value={text()}
          onInput={(e) => props.onDraft(e.currentTarget.value)}
        />
      }
    >
      <Match when={props.kind.type === 'bool'}>
        <span class={ux.checkboxRow}>
          <input
            id={props.wiring.id}
            type="checkbox"
            aria-describedby={props.wiring.describedBy}
            disabled={props.disabled}
            checked={props.draft === true}
            onChange={(e) => props.onDraft(e.currentTarget.checked)}
          />
          <span>{props.checkboxLabel ?? (props.draft === true ? t('admin-settings-value-on') : t('admin-settings-value-off'))}</span>
        </span>
      </Match>
      <Match when={props.kind.type === 'int' || props.kind.type === 'float'}>
        <input
          id={props.wiring.id}
          class={ux.control}
          type="number"
          inputmode={props.kind.type === 'int' ? 'numeric' : 'decimal'}
          min={props.kind.min}
          max={props.kind.max}
          step={props.kind.type === 'int' ? 1 : 'any'}
          aria-describedby={props.wiring.describedBy}
          aria-invalid={props.wiring.invalid}
          disabled={props.disabled}
          value={text()}
          onInput={(e) => props.onDraft(e.currentTarget.value)}
        />
      </Match>
      <Match when={props.kind.type === 'enum' || props.kind.type === 'locale'}>
        <select
          id={props.wiring.id}
          class={ux.control}
          aria-describedby={props.wiring.describedBy}
          aria-invalid={props.wiring.invalid}
          disabled={props.disabled}
          value={text()}
          onChange={(e) => props.onDraft(e.currentTarget.value)}
        >
          {props.nullable ? <option value="">{t('admin-settings-value-empty')}</option> : null}
          <For each={options()}>{(v) => <option value={v}>{label(v)}</option>}</For>
        </select>
      </Match>
      <Match when={props.kind.type==='messageTemplates'}>
        <TemplateLibraryEditor value={(()=>{try{return templateRows(JSON.parse(text()));}catch{return [];}})()} onChange={rows=>props.onDraft(JSON.stringify(rows))} disabled={props.disabled===true} id={props.wiring.id} describedBy={props.wiring.describedBy}/>
      </Match>
      <Match when={isJsonKind(props.kind)}>
        <textarea
          id={props.wiring.id}
          class={ux.control}
          rows={6}
          spellcheck={false}
          aria-describedby={props.wiring.describedBy}
          aria-invalid={props.wiring.invalid}
          disabled={props.disabled}
          value={text()}
          onInput={(e) => props.onDraft(e.currentTarget.value)}
        />
      </Match>
    </Switch>
  );
}
