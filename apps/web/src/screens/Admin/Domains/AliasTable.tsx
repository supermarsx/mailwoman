import {For,Show,type JSX} from 'solid-js';
import {Icon} from '../../../components/Icon.tsx';
import {t} from '../../../i18n';
import * as css from './pages.css.ts';

export function AliasTable(props:{aliases:readonly string[];canWrite:boolean;busy:boolean;onEdit:(alias:string)=>void;onClone:(alias:string)=>void;onRemove:(alias:string)=>void}):JSX.Element {
  return <Show when={props.aliases.length} fallback={<p>{t('domain-editor-none')}</p>}>
    <table class={css.aliasTable} aria-label={t('domain-editor-aliases')}>
      <thead><tr><th scope="col">{t('domain-alias-name')}</th><Show when={props.canWrite}><th scope="col" class={css.aliasActionCell}>{t('domain-list-actions')}</th></Show></tr></thead>
      <tbody><For each={props.aliases}>{alias=><tr>
        <th scope="row" class={css.aliasName}>{alias}</th>
        <Show when={props.canWrite}><td class={css.aliasActionCell}><div class={css.aliasActions}>
          <button type="button" class={`btn btn--ghost ${css.aliasAction}`} disabled={props.busy} aria-label={t('domain-alias-edit')} title={t('domain-alias-edit')} onClick={()=>props.onEdit(alias)}><Icon name="edit"/></button>
          <button type="button" class={`btn btn--ghost ${css.aliasAction}`} disabled={props.busy} aria-label={t('domain-alias-clone')} title={t('domain-alias-clone')} onClick={()=>props.onClone(alias)}><Icon name="clone"/></button>
          <button type="button" class={`btn btn--ghost ${css.aliasAction}`} disabled={props.busy} aria-label={t('domain-editor-remove-mapping',{value:alias})} title={t('domain-editor-remove')} onClick={()=>props.onRemove(alias)}><Icon name="delete"/></button>
        </div></td></Show>
      </tr>}</For></tbody>
    </table>
  </Show>;
}
