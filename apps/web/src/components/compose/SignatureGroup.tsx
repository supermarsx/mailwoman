// The ribbon's Signature control: a split button (insert / remove the current
// signature) with a menu to choose, replace, copy at the caret, set the
// position, create and edit. Insert tab, group "Signature".

import { createSignal, For, Show, type Accessor, type JSX } from 'solid-js';
import { RibbonGroup, RibbonItem } from '../bars/index.ts';
import { Icon, type IconName } from '../Icon.tsx';
import { RibbonPopover } from './ribbonParts.tsx';
import { useComposeFeature } from './composeFeatures.ts';
import { t, isolate } from '../../i18n/index.ts';
import * as a11y from '../mailA11y.css.ts';
import type { ComposeSignature } from './ComposerExtras.tsx';
import type { SignatureController } from './signatureControl.ts';
import { SIGNATURE_POSITIONS, type SignaturePosition } from './signatureRegion.ts';
import * as css from './signatureGroup.css.ts';
import { loadCatalog } from '../../i18n/catalog.ts';
// i18n: the ids this file shows live in this catalog, which loads with it.
void loadCatalog('compose-signature');

export interface SignatureGroupItem { sig: ComposeSignature; isDefault: boolean }
export interface SignatureMenuGroup { identityId: string; name: string; current: boolean; items: SignatureGroupItem[] }

export interface SignatureGroupProps {
  controller: SignatureController;
  groups: Accessor<SignatureMenuGroup[]>;
  disabled: Accessor<boolean>;
  onEdit: (mode: 'new' | 'manage', returnTo: HTMLElement | null) => void;
  onError?: (message: string) => void;
}

const POSITION_LABEL: Record<SignaturePosition, string> = {
  'above-quote': 'sig-position-above',
  'below-quote': 'sig-position-below',
  'end': 'sig-position-end',
};

const previewOf = (sig: ComposeSignature): string => (sig.text || '').replace(/\s+/g, ' ').trim().slice(0, 70);

export function SignatureGroup(props: SignatureGroupProps): JSX.Element {
  const on = useComposeFeature('insert.signature');
  const canEditFlag = useComposeFeature('insert.signatureEdit');
  const canPosition = useComposeFeature('options.signaturePosition');
  const c = props.controller;
  const [open, setOpen] = createSignal(false);
  const editAllowed = (): boolean => canEditFlag();
  const fail = (): void => props.onError?.(t('sig-failed'));
  const manage=(event:MouseEvent):void=>{
    const group=(event.currentTarget as HTMLElement).closest('[data-ribbon-group]');
    const target=group?.querySelector<HTMLButtonElement>(group.getAttribute('data-folded')==='true'?'[data-ribbon-more]':'[data-testid=signature-menu]')??null;
    setOpen(false);props.onEdit('manage',target);
  };
  const run = (work: () => Promise<unknown> | void): void => {
    setOpen(false);
    void Promise.resolve().then(work).catch(fail);
  };
  const all = (): SignatureGroupItem[] => props.groups().flatMap((g) => g.items);
  const item = (testId: string, icon: IconName, text: string, onClick: (event: MouseEvent) => void, extra: { disabled?: boolean; checked?: boolean; role?: 'menuitem' | 'menuitemradio' } = {}): JSX.Element => (
    <button type="button" role={extra.role ?? 'menuitem'} aria-checked={extra.checked === undefined ? undefined : extra.checked}
      class={`compose-rt__menuitem ${a11y.focusable}`} data-testid={testId} disabled={extra.disabled === true || props.disabled() || c.busy()}
      onClick={onClick}>
      <Icon name={extra.checked ? 'check' : icon} /><span>{text}</span>
    </button>
  );
  return (
    <Show when={on()}>
      <RibbonGroup label={t('sig-group')} priority={40} icon="signature" testId="compose-group-signature">
        <RibbonItem>
          <div class={css.split} data-testid="compose-signature">
            <button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="signature-primary"
              aria-pressed={c.applied() ? 'true' : 'false'} disabled={props.disabled() || c.busy()}
              data-tooltip={c.applied() ? t('sig-remove') : t('sig-insert')}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => run(async () => { if (!(await c.toggle())) setOpen(true); })}>
              <Icon name={c.applied() ? 'check' : 'signature'} /><span class="compose-rib__text">{t('sig-group')}</span>
            </button>
            <RibbonPopover label={t('sig-options')} icon="chevronDown" iconOnly buttonClass={`compose-rt__btn ${a11y.focusable}`}
              buttonTestId="signature-menu" disabled={props.disabled()} open={open()} onOpenChange={setOpen}>
              <div class={`compose-rt__menu ${css.menu}`} role="menu" aria-label={t('sig-options')} data-testid="signature-menu-list">
                <For each={props.groups()}>{(group) => (
                  <section aria-label={isolate(group.name)}>
                    <h3 class={css.heading}>{isolate(group.name)}{group.current ? ` · ${t('sig-from-current')}` : ''}</h3>
                    <For each={group.items}>{(entry) => (
                      <button type="button" role="menuitemradio" aria-checked={c.applied()?.sig.id === entry.sig.id}
                        class={`compose-rt__menuitem ${a11y.focusable} ${css.choice}`} data-testid={`signature-choice-${entry.sig.id}`}
                        disabled={props.disabled() || c.busy()} onClick={() => run(() => c.apply(entry.sig))}>
                        <Icon name={c.applied()?.sig.id === entry.sig.id ? 'check' : 'signature'} />
                        <span>{isolate(entry.sig.name)}</span>
                        <Show when={entry.isDefault}><span class={css.badge}>{t('sig-default')}</span></Show>
                        <span class={css.preview}>{isolate(previewOf(entry.sig))}</span>
                      </button>
                    )}</For>
                  </section>
                )}</For>
                <Show when={all().length === 0}><p class={css.heading} data-testid="signature-none-available">{t('sig-none-available')}</p></Show>
                <Show when={c.offer()}>
                  <p class={css.heading} role="status">{t('sig-identity-changed')}</p>
                  {item('signature-accept-identity','check',t('sig-use-identity'),()=>run(()=>c.acceptOffer()))}
                  {item('signature-keep-edited','edit',t('sig-keep-edited'),()=>c.dismissOffer())}
                </Show>
                <hr class={css.rule} />
                {item('signature-none', 'minus', t('sig-no-signature'), () => run(() => { c.remove(); }))}
                <Show when={all().length > 0}>
                  {item('signature-at-cursor', 'duplicate', t('sig-at-cursor'), () => run(() => {
                    const entry = all().find((e) => e.sig.id === c.applied()?.sig.id) ?? all().find((e) => e.isDefault) ?? all()[0];
                    if (entry) return c.insertCopy(entry.sig);
                  }))}
                </Show>
                <Show when={canPosition()}>
                  <hr class={css.rule} />
                  <h3 class={css.heading} id="sig-pos-heading">{t('sig-position')}</h3>
                  <div role="group" aria-labelledby="sig-pos-heading">
                    <For each={SIGNATURE_POSITIONS}>{(value) =>
                      item(`signature-position-${value}`, 'minus', t(POSITION_LABEL[value]), () => run(() => c.reposition(value)), { role: 'menuitemradio', checked: c.position() === value })}</For>
                  </div>
                </Show>
                <Show when={editAllowed()}>
                  <hr class={css.rule} />
                  {item('signature-edit', 'edit', t('sig-edit'), manage)}
                </Show>
              </div>
            </RibbonPopover>
          </div>
        </RibbonItem>
      </RibbonGroup>
    </Show>
  );
}
