// The open message changed somewhere else (t39 L8): another device or app moved
// it, or it no longer exists. The reader keeps the copy it shows and its place;
// this line says what happened, without a dialog and without taking focus.
import { Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import type { MessageView } from '../state/slices/messageView.ts';
import { t, isolate } from '../i18n/index.ts';
import { Icon } from './Icon.tsx';
import * as a11y from './mailA11y.css.ts';
import * as css from './RemoteChangeNotice.css.ts';

export function RemoteChangeNotice(props: { view: MessageView }): JSX.Element {
  const app = useApp();
  const folders = (ids: Readonly<Record<string, boolean>>): string =>
    app
      .mailboxes()
      .filter((box) => ids[box.id] === true)
      .map((box) => box.name)
      .join(', ');
  const text = (): string => {
    const change = props.view.remoteChange();
    if (change === null) return '';
    if (change.kind === 'deleted') return t('mail-remote-deleted');
    const names = folders(change.mailboxIds);
    return names ? t('mail-remote-moved', { folder: isolate(names) }) : t('mail-remote-moved-unknown');
  };
  return (
    <div role="status" data-testid="reader-remote-change-region">
      <Show when={props.view.remoteChange() !== null}>
        <p class={`reader__remote-change ${css.notice}`} data-testid="reader-remote-change">
          <Icon name="alert" />
          <span>{text()}</span>
          <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => props.view.dismissRemoteChange()}>
            {t('mail-remote-dismiss')}
          </button>
        </p>
      </Show>
    </div>
  );
}
