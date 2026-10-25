// "Last changed by X at T" and the History link for a field or object (t32-cfg UX8).
//
// The link opens the audit log filtered to the object: `#/audit?target=<kind>:<id>`
// (target naming in t32-cfg §4.6 R-CFG 3: `setting:<key>`, `domain:<name>`,
// `integration:<kind>`, ...). Inside the console it navigates in place through
// the shell router (so the dirty-state guard is asked); a modified click opens a
// new tab as any link does.

import { Show, useContext, type JSX } from 'solid-js';
import { formatDateTime, isolate, t } from '../../../i18n';
import { ShellContext } from './context.ts';
import { hrefFor, sectionPath } from './router.ts';
import * as ux from './ux.css.ts';

export interface FieldHistoryProps {
  /** Audit target, `<kind>:<id>` (`setting:signin.newUsers`). */
  target: string;
  updatedBy?: string | null | undefined;
  /** ISO 8601. */
  updatedAt?: string | null | undefined;
  /** The audit section's id. Default `audit`. */
  auditSection?: string;
}

/** The console location of an object's audit history. */
export function historyPath(target: string, auditSection = 'audit'): string {
  return sectionPath(auditSection, [], { target });
}

export function FieldHistory(props: FieldHistoryProps): JSX.Element {
  const shell = useContext(ShellContext);
  const path = (): string => historyPath(props.target, props.auditSection);
  const changed = (): string => {
    const when = props.updatedAt ? formatDateTime(props.updatedAt) : '';
    if (props.updatedBy && when) return t('admin-ux-history-changed-by-at', { who: isolate(props.updatedBy), time: when });
    if (props.updatedBy) return t('admin-ux-history-changed-by', { who: isolate(props.updatedBy) });
    if (when) return t('admin-ux-history-changed-at', { time: when });
    return t('admin-ux-history-never');
  };
  function onClick(e: MouseEvent): void {
    if (!shell || e.defaultPrevented || e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    void shell.router.navigate(path());
  }
  return (
    <span class={ux.stateRow} data-testid="field-history">
      <span>{changed()}</span>
      <Show when={props.target !== ''}>
        <a class={ux.learnMore} href={hrefFor(path())} onClick={onClick}>
          {t('admin-ux-history-link')}
        </a>
      </Show>
    </span>
  );
}
