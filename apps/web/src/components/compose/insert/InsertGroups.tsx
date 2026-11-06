// The composer's Insert tab (Office style): Attach, Illustrations, Links, Cards and
// calendar, Symbols. The composer owns the draft; these groups call back through
// `InsertHost`. Every control is wrapped in its feature flag (`insert.*`), so a
// switched-off feature is not rendered.

import { createSignal, For, Show, type Accessor, type JSX } from 'solid-js';
import { RibbonGroup, RibbonItem } from '../../bars/index.ts';
import { Icon, type IconName } from '../../Icon.tsx';
import { RibbonPopover } from '../ribbonParts.tsx';
import { useComposeFeature } from '../composeFeatures.ts';
import { t } from '../../../i18n/index.ts';
import * as a11y from '../../mailA11y.css.ts';
import type { ContactCard } from '../../../api/pim-types.ts';
import type { AttachmentItem, AttachmentPage } from '../../../viewers/attachments.ts';
import type { CalendarController } from '../../../modules/calendar/controller.ts';
import { LinkDialog, type LinkInitial } from './LinkDialog.tsx';
import { BusinessCardDialog, type CardSource } from './BusinessCardDialog.tsx';
import { CalendarDialog, type CalendarKind } from './CalendarDialog.tsx';
import { EventDialog } from './EventDialog.tsx';
import { ShapeDialog } from './ShapeDialog.tsx';
import { MessagePicturesDialog } from './MessagePicturesDialog.tsx';
import { PictureFileDialog } from './PictureFileDialog.tsx';
import { SPECIAL_CHARACTERS } from './symbols.ts';
import type { ResizePreset } from './imageResize.ts';

/** What the Insert tab needs from the composer. */
export interface InsertHost {
  disabled: Accessor<boolean>;
  /** A block (card, table, rule) or inline run into the body, in whichever mode the body is in. */
  insertBlock(html: string, text: string): void;
  insertInline(html: string, text: string): void;
  /** Upload and attach. */
  attachFiles(files: File[]): Promise<void>;
  /** Upload and place in the body as `cid:` images (attaches instead in plain-text mode). */
  placeImages(files: File[], alt?: string): Promise<void>;
  identity: Accessor<{ name: string; email: string; organization?: string | null } | null>;
  /** The current text selection in the body, for the link caption. */
  selectedText(): string;
  contacts: Accessor<ContactCard[]>;
  loadContacts(): Promise<void>;
  calendar: Accessor<CalendarController>;
  loadMessagePictures(position: number): Promise<AttachmentPage>;
  fetchMessagePicture(item: AttachmentItem): Promise<File>;
  richMode: Accessor<boolean>;
  /** The account's maximum attachment size, in bytes. */
  maxBytes: Accessor<number>;
  /** Admin defaults for pictures. */
  maxInlineMB: Accessor<number>;
  resizeDefault: Accessor<ResizePreset>;
  /** Exposed so the keyboard shortcut and the toolbar open the same dialog. */
  linkRequest: Accessor<LinkInitial | null>;
  setLinkRequest(value: LinkInitial | null): void;
  screenshotSupported: boolean;
  captureScreenshot(): Promise<File | null>;
  report(message: string): void;
}

type Dialog =
  | { kind: 'card'; source: CardSource }
  | { kind: 'calendar'; mode: CalendarKind }
  | { kind: 'event'; existing: boolean }
  | { kind: 'shape' }
  | { kind: 'messagePictures'; as: 'inline' | 'attach' }
  | { kind: 'pictureFile'; files: File[] };

export function InsertGroups(props: { host: InsertHost; /** The composer's own attach controls (file, account items, cloud), shown first in the Attach group. */ attachItems?: () => JSX.Element; showAttach?: boolean }): JSX.Element {
  const host = props.host;
  const [dialog, setDialog] = createSignal<Dialog | null>(null);
  let anchor: HTMLElement | null = null;
  const open = (value: Dialog, event?: Event): void => {
    anchor = (event?.currentTarget as HTMLElement | null) ?? null;
    setDialog(value);
  };
  const close = (): void => { setDialog(null); };
  const on = {
    attachFile: useComposeFeature('insert.attachFile'),
    attachFromMessages: useComposeFeature('insert.attachFromMessages'),
    pictureFile: useComposeFeature('insert.pictureFile'),
    pictureFromMessages: useComposeFeature('insert.pictureFromMessages'),
    shapes: useComposeFeature('insert.shapes'),
    screenshot: useComposeFeature('insert.screenshot'),
    businessCard: useComposeFeature('insert.businessCard'),
    calendar: useComposeFeature('insert.calendar'),
    event: useComposeFeature('insert.event'),
    symbols: useComposeFeature('insert.symbols'),
    horizontalLine: useComposeFeature('insert.horizontalLine'),
    dateStamp: useComposeFeature('insert.dateStamp'),
    link: useComposeFeature('insert.link'),
  };
  const [menu, setMenu] = createSignal<string | null>(null);
  const menuProps = (id: string) => ({ open: menu() === id, onOpenChange: (value: boolean) => setMenu(value ? id : (menu() === id ? null : menu())) });
  const item = (testId: string, icon: IconName, label: string, run: (event: MouseEvent) => void, disabled = false): JSX.Element => (
    <button type="button" role="menuitem" class={`compose-rt__menuitem ${a11y.focusable}`} data-testid={testId} disabled={disabled || host.disabled()} onClick={(event) => { setMenu(null); run(event); }}>
      <Icon name={icon} /><span>{label}</span>
    </button>
  );
  const menuButton = (id: string, label: string, icon: IconName, children: JSX.Element, testId: string): JSX.Element => (
    <RibbonPopover label={label} icon={icon} buttonClass={`compose-rt__btn ${a11y.focusable}`} buttonTestId={testId} disabled={host.disabled()} {...menuProps(id)}>
      <div class="compose-rt__menu" role="menu" aria-label={label}>{children}</div>
    </RibbonPopover>
  );

  const cardSelf = (): CardSource => ({ kind: 'self' });
  return (
    <>
      <Show when={props.showAttach !== false}><RibbonGroup label={t('mail-compose-attachments')} priority={55} icon="attachment" testId="compose-attach">
        {props.attachItems?.()}
        <Show when={on.attachFromMessages()}>
          <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-attach-messages" disabled={host.disabled()} onClick={(event) => open({ kind: 'messagePictures', as: 'attach' }, event)}><Icon name="mail" /><span class="compose-rib__text">{t('ins-attach-messages')}</span></button></RibbonItem>
        </Show>
      </RibbonGroup></Show>

      <Show when={on.pictureFile() || on.pictureFromMessages() || on.shapes() || on.screenshot()}>
        <RibbonGroup label={t('ins-group-illustrations')} priority={56} icon="image" testId="insert-group-illustrations">
          <Show when={on.pictureFile()}>
            <RibbonItem>
              <label class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-picture-file" aria-disabled={host.disabled()}>
                <Icon name="image" /><span class="compose-rib__text">{t('ins-picture-file')}</span>
                <input type="file" accept="image/png,image/jpeg,image/gif,image/webp,image/svg+xml" multiple class={a11y.srOnly} aria-label={t('ins-picture-file')} disabled={host.disabled()}
                  onChange={(event) => { const input = event.currentTarget; const files = Array.from(input.files ?? []); anchor = input.parentElement; input.value = ''; if (files.length > 0) setDialog({ kind: 'pictureFile', files }); }} />
              </label>
            </RibbonItem>
          </Show>
          <Show when={on.pictureFromMessages()}>
            <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-picture-messages" disabled={host.disabled()} onClick={(event) => open({ kind: 'messagePictures', as: 'inline' }, event)}><Icon name="mail" /><span class="compose-rib__text">{t('ins-picture-messages')}</span></button></RibbonItem>
          </Show>
          <Show when={on.shapes()}>
            <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-shapes" disabled={host.disabled()} onClick={(event) => open({ kind: 'shape' }, event)}><Icon name="shapes" /><span class="compose-rib__text">{t('ins-shapes')}</span></button></RibbonItem>
          </Show>
          <Show when={on.screenshot() && host.screenshotSupported}>
            <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-screenshot" disabled={host.disabled()} onClick={() => void host.captureScreenshot().then((file) => { if (file) setDialog({ kind: 'pictureFile', files: [file] }); })}><Icon name="screenshot" /><span class="compose-rib__text">{t('ins-screenshot')}</span></button></RibbonItem>
          </Show>
        </RibbonGroup>
      </Show>

      <Show when={on.link()}>
        <RibbonGroup label={t('ins-group-links')} priority={54} icon="link" testId="insert-group-links">
          <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-link" data-tooltip={`${t('ins-link')} (Ctrl+K)`} aria-keyshortcuts="Control+K" disabled={host.disabled()} onClick={() => host.setLinkRequest({ caption: host.selectedText(), href: '', title: '', newTab: false, existing: false })}><Icon name="link" /><span class="compose-rib__text">{t('ins-link')}</span></button></RibbonItem>
        </RibbonGroup>
      </Show>

      <Show when={on.businessCard() || on.calendar() || on.event()}>
        <RibbonGroup label={t('ins-group-cards')} priority={52} icon="calendar" testId="insert-group-cards">
          <Show when={on.businessCard()}>
            <RibbonItem>{menuButton('card', t('ins-business-card'), 'idCard', <>
              {item('insert-card-self', 'person', t('ins-card-mine'), (event) => open({ kind: 'card', source: cardSelf() }, event))}
              {item('insert-card-contact', 'users', t('ins-card-contact'), (event) => { void host.loadContacts(); open({ kind: 'card', source: { kind: 'contact' } }, event); })}
            </>, 'insert-business-card')}</RibbonItem>
          </Show>
          <Show when={on.calendar()}>
            <RibbonItem>{menuButton('calendar', t('ins-calendar'), 'calendar', <>
              {item('insert-cal-availability', 'clock', t('ins-cal-availability'), (event) => open({ kind: 'calendar', mode: 'availability' }, event))}
              {item('insert-cal-agenda', 'bulletList', t('ins-cal-agenda'), (event) => open({ kind: 'calendar', mode: 'agenda' }, event))}
              {item('insert-cal-snapshot', 'calendar', t('ins-cal-snapshot'), (event) => open({ kind: 'calendar', mode: 'snapshot' }, event))}
            </>, 'insert-calendar')}</RibbonItem>
          </Show>
          <Show when={on.event()}>
            <RibbonItem>{menuButton('event', t('ins-event'), 'calendarPlus', <>
              {item('insert-event-existing', 'calendar', t('ins-event-existing'), (event) => open({ kind: 'event', existing: true }, event))}
              {item('insert-event-new', 'plus', t('ins-event-new'), (event) => open({ kind: 'event', existing: false }, event))}
            </>, 'insert-event')}</RibbonItem>
          </Show>
        </RibbonGroup>
      </Show>

      <Show when={on.symbols() || on.horizontalLine() || on.dateStamp()}>
        <RibbonGroup label={t('ins-group-symbols')} priority={50} icon="smile" testId="insert-group-symbols">
          <Show when={on.symbols()}>
            <RibbonItem>
              <RibbonPopover label={t('ins-symbols')} icon="smile" buttonClass={`compose-rt__btn ${a11y.focusable}`} buttonTestId="insert-symbols" disabled={host.disabled()} {...menuProps('symbols')}>
                <div class="compose-rt__symbols" role="group" aria-label={t('ins-symbols')} style={{ display: 'grid', 'grid-template-columns': 'repeat(8, 2rem)', gap: '2px', padding: '4px' }}>
                  <For each={SPECIAL_CHARACTERS}>{(entry) => (
                    <button type="button" class={`compose-rt__btn ${a11y.focusable}`} aria-label={entry.name} data-tooltip={entry.name} data-testid="insert-symbol" onMouseDown={(event) => event.preventDefault()} onClick={() => { host.insertInline(entry.ch, entry.ch); setMenu(null); }}>{entry.ch}</button>
                  )}</For>
                </div>
              </RibbonPopover>
            </RibbonItem>
          </Show>
          <Show when={on.horizontalLine()}>
            <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-hr" disabled={host.disabled()} onMouseDown={(event) => event.preventDefault()} onClick={() => host.insertBlock('<hr>', '\n----------\n')}><Icon name="minus" /><span class="compose-rib__text">{t('ins-hr')}</span></button></RibbonItem>
          </Show>
          <Show when={on.dateStamp()}>
            <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="insert-date" disabled={host.disabled()} onMouseDown={(event) => event.preventDefault()} onClick={() => { const text = new Intl.DateTimeFormat(undefined, { dateStyle: 'long', timeStyle: 'short' }).format(new Date()); host.insertInline(text, text); }}><Icon name="clock" /><span class="compose-rib__text">{t('ins-date')}</span></button></RibbonItem>
          </Show>
        </RibbonGroup>
      </Show>

      <Show when={host.linkRequest()} keyed>
        {(initial) => <LinkDialog initial={initial} richMode={host.richMode()} returnTo={anchor} onClose={() => host.setLinkRequest(null)} onInsert={(html, text) => { host.setLinkRequest(null); host.insertInline(html, text); }} />}
      </Show>
      <Show when={dialog()} keyed>
        {(current) => {
          switch (current.kind) {
            case 'card':
              return <BusinessCardDialog source={current.source} identity={host.identity()} contacts={host.contacts()} returnTo={anchor} onClose={close} onInsert={(html, text, vcf) => { close(); if (html !== null) host.insertBlock(html, text); if (vcf) void host.attachFiles([new File([vcf.text], vcf.name, { type: vcf.type })]); }} />;
            case 'calendar':
              return <CalendarDialog mode={current.mode} controller={host.calendar()} returnTo={anchor} onClose={close} onInsert={(html, text) => { close(); host.insertBlock(html, text); }} />;
            case 'event':
              return <EventDialog existing={current.existing} controller={host.calendar()} identity={host.identity()} returnTo={anchor} onClose={close} onInsert={(html, text, ics) => { close(); host.insertBlock(html, text); void host.attachFiles([new File([ics.text], 'invite.ics', { type: `text/calendar; method=${ics.method}` })]); }} />;
            case 'shape':
              return <ShapeDialog returnTo={anchor} onClose={close} onInsert={(file, alt) => { close(); void host.placeImages([file], alt); }} />;
            case 'messagePictures':
              return <MessagePicturesDialog as={current.as} loadPage={host.loadMessagePictures} fetchPicture={host.fetchMessagePicture} returnTo={anchor} onClose={close} onPick={(files) => { close(); if (current.as === 'inline') void host.placeImages(files); else void host.attachFiles(files); }} />;
            case 'pictureFile':
              return <PictureFileDialog files={current.files} defaultPreset={host.resizeDefault()} maxBytes={host.maxInlineMB() * 1024 * 1024} returnTo={anchor} onClose={close} onInsert={(files, alt, asAttachment) => { close(); if (asAttachment) void host.attachFiles(files); else void host.placeImages(files, alt === '' ? undefined : alt); }} />;
          }
        }}
      </Show>
    </>
  );
}
