// t86: change a mailbox's avatar (Settings › Linked mailboxes). The avatar in
// the row is the trigger; it opens an anchored popover (top layer where the
// browser has it, placed by components/color/placement.ts like the colour
// picker) with a live preview, a Letter/Icon choice, the icon grid (a radio
// group: arrows move, tooltips name each icon) or the letter field, the
// foreground and background colours (the shared ColorField), and a contrast
// note: Auto picks black or white for AA against the background, and a manual
// foreground below AA is called out with a one-click way back to Auto.
//
// Keyboard: Escape cancels and returns focus to the avatar; Tab stays inside.
// Nothing is saved until Save; Reset goes back to the generated avatar.
import { createSignal, For, onCleanup, Show, type JSX } from 'solid-js';
import { MailboxAvatar } from '../../components/accounts/MailboxPicker.tsx';
import { ColorField } from '../../components/color/ColorPicker.tsx';
import { placePopover } from '../../components/color/placement.ts';
import { Icon } from '../../components/Icon.tsx';
import { isolate, t } from '../../i18n/index.ts';
import {
  AVATAR_CONTRAST_MIN, AVATAR_ICONS, AVATAR_TEXT_MAX, avatarIconLabel, avatarLook, cleanAvatarText, defaultAvatarText,
  type AvatarIconId, type MailboxAvatarSpec,
} from '../../api/mailboxAvatar.ts';
import * as css from './styles.css.ts';
import * as ed from './avatarEditor.css.ts';

const popoverSupported = typeof HTMLElement !== 'undefined' && typeof (HTMLElement.prototype as { showPopover?: unknown }).showPopover === 'function';

export interface MailboxAvatarEditorProps {
  /** The layout key (`main` or a link id), for test hooks. */
  mailboxKey: string;
  label: string;
  address: string;
  avatar: MailboxAvatarSpec | undefined;
  disabled?: boolean;
  /** Save (`undefined` = back to the generated avatar); resolves false when refused. */
  onSave: (avatar: MailboxAvatarSpec | undefined) => Promise<boolean>;
}

export function MailboxAvatarEditor(props: MailboxAvatarEditorProps): JSX.Element {
  const [open, setOpen] = createSignal(false);
  const [kind, setKind] = createSignal<'letter' | 'icon'>('letter');
  const [text, setText] = createSignal('');
  const [icon, setIcon] = createSignal<AvatarIconId>('briefcase');
  const [fg, setFg] = createSignal<string | undefined>();
  const [bg, setBg] = createSignal<string | undefined>();
  const [error, setError] = createSignal('');
  const [saving, setSaving] = createSignal(false);
  let trigger: HTMLButtonElement | undefined;
  let pop: HTMLDivElement | undefined;
  let detach: (() => void) | null = null;
  const titleId = `mw-avatar-editor-${props.mailboxKey}`;
  const letterId = `${titleId}-letter`;
  const name = (): string => (props.label === props.address ? props.address : `${props.label} (${props.address})`);

  const textValue = (): string | null => cleanAvatarText(text());
  /** The draft as it would be saved; `undefined` = the generated avatar. */
  const draft = (): MailboxAvatarSpec | undefined => {
    const colours = { ...(fg() ? { fg: fg()! } : {}), ...(bg() ? { bg: bg()! } : {}) };
    if (kind() === 'icon') return { kind: 'icon', icon: icon(), ...colours };
    const value = textValue() ?? '';
    if (!value && !fg() && !bg()) return undefined;
    return { kind: 'letter', ...(value ? { text: value } : {}), ...colours };
  };
  // The preview always shows the custom look while editing (so Auto is visible).
  const look = (): ReturnType<typeof avatarLook> => avatarLook(draft() ?? { kind: 'letter' }, props.label, props.address);
  const lowContrast = (): boolean => !!fg() && look().ratio < AVATAR_CONTRAST_MIN;

  function view(): Window { return trigger?.ownerDocument.defaultView ?? window; }
  function place(): void {
    if (!trigger || !pop || !open()) return;
    const w = view();
    const vv = w.visualViewport;
    const anchor = trigger.getBoundingClientRect();
    const spot = placePopover(
      { left: anchor.left, top: anchor.top, width: anchor.width, height: anchor.height },
      { width: pop.offsetWidth, height: pop.scrollHeight },
      { left: vv?.offsetLeft ?? 0, top: vv?.offsetTop ?? 0, width: vv?.width ?? w.innerWidth, height: vv?.height ?? w.innerHeight },
      { rtl: w.getComputedStyle(trigger).direction === 'rtl' },
    );
    // CSSOM only (shell CSP: no style attribute).
    pop.style.setProperty('left', `${spot.left}px`);
    pop.style.setProperty('top', `${spot.top}px`);
    pop.style.setProperty('max-height', `${spot.maxHeight}px`);
  }

  function show(): void {
    if (props.disabled || open()) return;
    const saved = props.avatar;
    setKind(saved?.kind ?? 'letter');
    setText(saved?.kind === 'letter' ? saved.text ?? '' : '');
    setIcon(saved?.kind === 'icon' && saved.icon ? saved.icon : 'briefcase');
    setFg(saved?.fg);
    setBg(saved?.bg);
    setError('');
    setOpen(true);
    if (pop && popoverSupported && pop.hasAttribute('popover')) {
      try { pop.showPopover(); } catch { /* not connected or already shown */ }
    }
    place();
    const w = view();
    w.requestAnimationFrame(() => {
      place();
      pop?.querySelector<HTMLElement>('[aria-pressed="true"]')?.focus({ preventScroll: true });
    });
    const d = trigger!.ownerDocument;
    const outside = (event: PointerEvent): void => {
      const target = event.target as Node | null;
      if (target && (pop?.contains(target) || trigger?.contains(target))) return;
      hide(false);
    };
    const follow = (): void => place();
    d.addEventListener('pointerdown', outside, true);
    d.addEventListener('scroll', follow, true);
    w.addEventListener('resize', follow);
    w.visualViewport?.addEventListener('resize', follow);
    detach = () => {
      d.removeEventListener('pointerdown', outside, true);
      d.removeEventListener('scroll', follow, true);
      w.removeEventListener('resize', follow);
      w.visualViewport?.removeEventListener('resize', follow);
    };
  }
  function hide(restoreFocus = true): void {
    if (!open()) return;
    detach?.();
    detach = null;
    if (pop && popoverSupported && pop.hasAttribute('popover')) {
      try { pop.hidePopover(); } catch { /* already hidden */ }
    }
    setOpen(false);
    if (restoreFocus) trigger?.focus({ preventScroll: true });
  }
  onCleanup(() => detach?.());

  async function save(value: MailboxAvatarSpec | undefined): Promise<void> {
    if (kind() === 'letter' && textValue() === null) { setError(t('linked-accounts-avatar-letter-invalid', { max: AVATAR_TEXT_MAX })); return; }
    setSaving(true);
    setError('');
    try {
      if (await props.onSave(value)) hide();
      else setError(t('linked-accounts-avatar-failed'));
    } finally { setSaving(false); }
  }

  function focusables(): HTMLElement[] {
    return Array.from(pop?.querySelectorAll<HTMLElement>('button:not([disabled]):not([tabindex="-1"]),input:not([disabled])') ?? [])
      .filter((el) => !el.closest('[data-color-popover]') && el.offsetParent !== null);
  }
  function onKeyDown(event: KeyboardEvent): void {
    if (event.defaultPrevented) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); hide(); return; }
    if (event.key !== 'Tab') return;
    const list = focusables();
    if (list.length === 0) return;
    const at = list.indexOf(event.target as HTMLElement);
    if (event.shiftKey && at <= 0) { event.preventDefault(); list[list.length - 1]!.focus(); }
    else if (!event.shiftKey && at === list.length - 1) { event.preventDefault(); list[0]!.focus(); }
  }

  /** Arrows move through the icon grid (rows by the rendered column count). */
  function onGridKey(event: KeyboardEvent): void {
    const cells = Array.from(pop?.querySelectorAll<HTMLButtonElement>('[data-avatar-choice]') ?? []);
    const at = cells.indexOf(event.target as HTMLButtonElement);
    if (at < 0) return;
    const columns = Math.max(1, cells.filter((cell) => cell.offsetTop === cells[0]!.offsetTop).length);
    const rtl = view().getComputedStyle(cells[0]!).direction === 'rtl';
    const step: Record<string, number> = { ArrowRight: rtl ? -1 : 1, ArrowLeft: rtl ? 1 : -1, ArrowDown: columns, ArrowUp: -columns, Home: -at, End: cells.length - 1 - at };
    const move = step[event.key];
    if (move === undefined) return;
    event.preventDefault();
    const next = Math.min(cells.length - 1, Math.max(0, at + move));
    const id = cells[next]!.dataset['avatarChoice'] as AvatarIconId;
    setIcon(id);
    cells[next]!.focus();
  }

  return (
    <span class={ed.wrap}>
      <button
        ref={trigger}
        type="button"
        class={ed.avatarButton}
        data-linked-avatar
        aria-haspopup="dialog"
        aria-expanded={open() ? 'true' : 'false'}
        aria-label={t('linked-accounts-avatar-change-for', { name: isolate(name()) })}
        data-tooltip={t('linked-accounts-avatar-change')}
        disabled={props.disabled}
        onClick={() => (open() ? hide() : show())}
      >
        <MailboxAvatar label={props.label} address={props.address} avatar={props.avatar} />
      </button>
      <div
        ref={(el) => { pop = el; if (popoverSupported) el.setAttribute('popover', 'manual'); }}
        class={ed.popover}
        role="dialog"
        aria-modal="false"
        aria-labelledby={titleId}
        hidden={!open()}
        data-avatar-editor={props.mailboxKey}
        onKeyDown={onKeyDown}
      >
        <Show when={open()}>
          <div class={ed.head}>
            <span data-avatar-preview><MailboxAvatar label={props.label} address={props.address} avatar={draft() ?? { kind: 'letter' }} size="lg" /></span>
            <div class={ed.headText}>
              <p id={titleId} class={ed.title}>{t('linked-accounts-avatar-title')}</p>
              <p class={ed.subtitle}>{isolate(name())}</p>
            </div>
          </div>
          <div class={css.options} role="group" aria-label={t('linked-accounts-avatar-kind')}>
            <button type="button" class={css.option} aria-pressed={kind() === 'letter' ? 'true' : 'false'} data-avatar-mode="letter" onClick={() => setKind('letter')}>{t('linked-accounts-avatar-letter')}</button>
            <button type="button" class={css.option} aria-pressed={kind() === 'icon' ? 'true' : 'false'} data-avatar-mode="icon" onClick={() => setKind('icon')}>{t('linked-accounts-avatar-icon')}</button>
          </div>
          <Show when={kind() === 'icon'} fallback={
            <div class={ed.field}>
              <label class={ed.label} for={letterId}>{t('linked-accounts-avatar-letter-label')}</label>
              <input id={letterId} class={`${css.input} ${ed.letter}`} type="text" data-avatar-letter value={text()} placeholder={defaultAvatarText(props.label, props.address)}
                maxLength={8} autocomplete="off" spellcheck={false} aria-describedby={`${letterId}-help`} aria-invalid={textValue() === null ? 'true' : undefined}
                onInput={(event) => setText(event.currentTarget.value)}
                onKeyDown={(event) => { if (event.key === 'Enter') { event.preventDefault(); void save(draft()); } }} />
              <p id={`${letterId}-help`} class={textValue() === null ? css.error : ed.note}>
                {textValue() === null ? t('linked-accounts-avatar-letter-invalid', { max: AVATAR_TEXT_MAX }) : t('linked-accounts-avatar-letter-help', { max: AVATAR_TEXT_MAX })}
              </p>
            </div>}>
            <div class={ed.grid} role="radiogroup" aria-label={t('linked-accounts-avatar-icon-choose')} data-avatar-icons onKeyDown={onGridKey}>
              <For each={AVATAR_ICONS}>{(entry) => (
                <button type="button" role="radio" class={ed.cell} data-avatar-choice={entry.id}
                  aria-checked={icon() === entry.id ? 'true' : 'false'} tabIndex={icon() === entry.id ? 0 : -1}
                  aria-label={t(avatarIconLabel(entry.id))} data-tooltip={t(avatarIconLabel(entry.id))}
                  onClick={() => setIcon(entry.id)}>
                  <Icon name={entry.glyph} />
                </button>
              )}</For>
            </div>
          </Show>
          <div class={ed.colours}>
            <div class={ed.field} data-avatar-bg>
              <span class={ed.label}>{t('linked-accounts-avatar-background')}</span>
              <ColorField value={bg() ?? null} label={t('linked-accounts-avatar-background')} emptyLabel={t('linked-accounts-avatar-default')}
                fallback={look().bg} onPreview={(value) => setBg(value ?? undefined)} onChange={(value) => setBg(value ?? undefined)} testId={`avatar-bg-${props.mailboxKey}`} />
            </div>
            <div class={ed.field} data-avatar-fg>
              <span class={ed.label}>{t(kind() === 'icon' ? 'linked-accounts-avatar-icon-colour' : 'linked-accounts-avatar-letter-colour')}</span>
              <ColorField value={fg() ?? null} label={t(kind() === 'icon' ? 'linked-accounts-avatar-icon-colour' : 'linked-accounts-avatar-letter-colour')}
                emptyLabel={t('linked-accounts-avatar-auto')} fallback={look().fg}
                contrast={{ against: () => look().bg, use: 'text', againstLabel: t('linked-accounts-avatar-background-lower') }}
                onPreview={(value) => setFg(value ?? undefined)} onChange={(value) => setFg(value ?? undefined)} testId={`avatar-fg-${props.mailboxKey}`} />
            </div>
          </div>
          <Show when={lowContrast()} fallback={
            <p class={ed.note} data-avatar-contrast="ok">
              {look().autoFg ? t('linked-accounts-avatar-auto-note', { ratio: look().ratio.toFixed(1) }) : t('linked-accounts-avatar-contrast-ok', { ratio: look().ratio.toFixed(1) })}
            </p>}>
            <p class={ed.warn} role="status" data-avatar-contrast="low">
              <Icon name="alert" />
              <span>{t('linked-accounts-avatar-contrast-low', { ratio: look().ratio.toFixed(1) })}</span>
              <button type="button" class={css.ghost} data-avatar-use-auto onClick={() => setFg(undefined)}>{t('linked-accounts-avatar-use-auto')}</button>
            </p>
          </Show>
          <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
          <div class={ed.footer}>
            <button type="button" class={css.ghost} data-avatar-reset disabled={saving() || !props.avatar} onClick={() => void save(undefined)}>{t('linked-accounts-avatar-reset')}</button>
            <span class={ed.spacer} />
            <button type="button" class={css.ghost} data-avatar-cancel onClick={() => hide()}>{t('linked-accounts-cancel')}</button>
            <button type="button" class={css.button} data-avatar-save disabled={saving() || (kind() === 'letter' && textValue() === null)} onClick={() => void save(draft())}>{t('linked-accounts-avatar-save')}</button>
          </div>
        </Show>
      </div>
    </span>
  );
}
