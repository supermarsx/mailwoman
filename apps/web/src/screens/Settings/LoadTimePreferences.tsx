// Per-person switches for the folder and message load-time measurements
// (perf/clientTiming.ts). Both are on by default; an administrator can set the
// default or lock it, and a locked switch shows the "set by your administrator" note.
// Turning one off stops the beacon at once, with no reload.

import { onMount, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { KIND_SETTING, type TimingKind } from '../../perf/clientTiming.ts';
import * as css from './styles.css.ts';

function KindSwitch(props: { kind: TimingKind }): JSX.Element {
  const setting = useSetting<boolean>(KIND_SETTING[props.kind], true);
  const help = `load-time-${props.kind}-help`;
  return <div>
    <label>
      <input type="checkbox" data-testid={`load-time-${props.kind}`} aria-describedby={help}
        checked={setting.value()} disabled={!setting.entry() || setting.locked() || setting.saving()}
        onChange={event => void setting.set(event.currentTarget.checked)} /> {t(`privacy-load-time-${props.kind}`)}
    </label>
    <p class={css.meta} id={help}>{t(`privacy-load-time-${props.kind}-help`)}</p>
    <LockNote locked={setting.locked()} />
    <Show when={setting.error()}><p role="alert">{setting.error()}</p></Show>
  </div>;
}

export function LoadTimePreferences(): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); });
  return <section class={css.section} aria-labelledby="load-time-heading" data-testid="load-time-preferences">
    <h3 id="load-time-heading" class={css.heading}>{t('privacy-load-time-title')}</h3>
    <p class={css.prose}>{t('privacy-load-time-help')}</p>
    <KindSwitch kind="folder" />
    <KindSwitch kind="message" />
  </section>;
}
