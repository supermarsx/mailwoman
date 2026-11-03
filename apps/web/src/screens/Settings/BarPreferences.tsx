// Settings → Appearance → Toolbars (t38-e2): pin or auto-hide each of the four
// collapsible bars. One radio pair per bar, stored in the settings registry
// (`ui.bar.<id>`, see `api/barSettings.ts`), so an administrator's default
// applies until the user picks and a locked bar shows "Set by your
// administrator" instead of an editable choice.

import {createUniqueId,For, onMount, Show, type JSX} from 'solid-js';
import {Icon} from '../../components/Icon.tsx';
import {loadCatalog, t} from '../../i18n';
import {BAR_IDS, BAR_MODE_DEFAULT, BAR_MODES, barSettingKey, isBarMode, type BarId, type BarMode} from '../../api/barSettings.ts';
import {useSetting} from './useSetting.ts';
import {useEffectiveBarMode} from '../../components/bars/useBarMode.ts';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';
import * as barCss from './BarPreferences.css.ts';

const BAR_LABEL: Readonly<Record<BarId, string>> = {
  mailTopBar: 'settings-bars-mail-top-bar',
  workspaceTabs: 'settings-bars-workspace-tabs',
  mailRibbon: 'settings-bars-mail-ribbon',
  composerRibbon: 'settings-bars-composer-ribbon',
};

const MODE_LABEL: Readonly<Record<BarMode, string>> = {
  pinned: 'settings-bars-mode-pinned',
  auto: 'settings-bars-mode-auto',
};

function BarChoice(props: {id: BarId}): JSX.Element {
  const setting = useSetting<BarMode>(barSettingKey(props.id), BAR_MODE_DEFAULT, isBarMode);
  // What the bar shows, including the narrow-screen default (t39 D4).
  const shown = useEffectiveBarMode(props.id, setting.entry);
  const name = `settings-bar-${props.id}-${createUniqueId()}`;
  const lockId = `${name}-lock`;
  const errorId = `${name}-error`;
  const describedBy = (): string | undefined =>
    [setting.locked() ? lockId : '', setting.error() ? errorId : ''].filter(Boolean).join(' ') || undefined;
  const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
  const buttons=new Map<BarMode,HTMLButtonElement>();
  function choose(mode:BarMode):void {if(!disabled()&&shown()!==mode)void setting.set(mode);}
  function navigate(event:KeyboardEvent,mode:BarMode):void {
    if(disabled()||!['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End'].includes(event.key))return;
    event.preventDefault();
    const next=event.key==='Home'?BAR_MODES[0]:event.key==='End'?BAR_MODES[BAR_MODES.length-1]:BAR_MODES[(BAR_MODES.indexOf(mode)+1)%BAR_MODES.length];
    if(next){buttons.get(next)?.focus();choose(next);}
  }
  return (
    <fieldset class={`${css.field} ${barCss.group}`} data-bar={props.id} aria-describedby={describedBy()}>
      <legend id={`${name}-label`} class={css.label}>{t(BAR_LABEL[props.id])}</legend>
      <div class={barCss.choices} role="radiogroup" aria-labelledby={`${name}-label`}>
        <For each={BAR_MODES}>
          {(mode) => (
            <button ref={element=>buttons.set(mode,element)} type="button" class={`btn btn--ghost ${barCss.choice}`} role="radio" aria-label={t(MODE_LABEL[mode])} aria-checked={shown()===mode} data-tooltip={t(MODE_LABEL[mode])} tabindex={shown()===mode?0:-1} disabled={disabled()} onClick={()=>choose(mode)} onKeyDown={event=>navigate(event,mode)}><Icon name={mode==='pinned'?'pin':'eye'}/></button>
          )}
        </For>
      </div>
      <LockNote id={lockId} locked={setting.locked()} />
      <Show when={setting.error()}>
        <p class={css.error} id={errorId} role="alert">{setting.error()}</p>
      </Show>
    </fieldset>
  );
}

export function BarPreferences(): JSX.Element {
  onMount(() => void loadCatalog('bars'));
  return (
    <section class={css.section} aria-labelledby="settings-bars-heading" data-testid="settings-bars">
      <h3 class={css.heading} id="settings-bars-heading">{t('settings-bars-heading')}</h3>
      <p class={css.meta}>{t('settings-bars-help')}</p>
      <For each={BAR_IDS}>{(id) => <BarChoice id={id} />}</For>
    </section>
  );
}
