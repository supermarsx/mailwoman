// The reader's notice bar (t110): one presentational frame for every notice above
// the message body, so they share one height, padding, radius, font, icon slot,
// tooltip and action-button style. The remote-images banner (t55) is the model;
// the mailing-list, linked-files and attachments bars use the same frame.
//
// Order in the reader, top to bottom (Reader.tsx renders them in one stack):
//   1. mailing list   2. remote images   3. linked files   4. attachments
// The attachments bar is last because its expanded region (the thumbnails) sits
// right above the body it belongs to.
//
// One row, always: [icon] [summary, ellipsised; its tooltip holds the full text
// and the explanation] [actions]. In a narrow container (the 390 px phone) the
// secondary action labels become visually hidden. Primary consent keeps its
// icon and label; the combined privacy segments can wrap on narrow screens.

import { Show, type JSX } from 'solid-js';
import { Icon, type IconName } from './Icon.tsx';
import { Spinner } from './spinner/Spinner.tsx';
import * as css from './reader-notice-bar.css.ts';

export { css as noticeCss };

export interface ReaderNoticeBarProps {
  /** The section's accessible name. */
  label: string;
  testid: string;
  /** Free-form state for tests and styling hooks (`data-state`). */
  state?: string;
  /** `warning` adds the warning rule and icon colour (remote images blocked). */
  tone?: 'warning' | 'neutral';
  icon: IconName;
  /** Full text plus explanation, shown in the summary's tooltip. */
  tooltip: string;
  /** Announce summary changes politely (the remote-images bar's "Images shown"). */
  live?: boolean;
  /** The summary content (headline and detail spans). */
  children: JSX.Element;
  actions?: JSX.Element;
  /** Rendered inside the frame after the row: a menu portal, an expanded region. */
  after?: JSX.Element;
  /** A segment of the single combined message privacy row. */
  embedded?: boolean;
  /** Makes the whole row a toggle target (the attachments bar). Clicks that land
   *  on a button, link or field inside the row are left to that control, and a
   *  click that ends a text selection does not toggle. The summary also toggles
   *  on Enter and Space; the real, announced button stays in `actions`. */
  onToggle?: () => void;
}

const INNER_CONTROL = 'button, a, input, select, textarea, [role="button"], [role="menuitem"], [data-no-toggle]';

function rowClick(event: MouseEvent, toggle: () => void): void {
  const target = event.target as Element | null;
  if (target?.closest(INNER_CONTROL)) return;
  const selection = globalThis.getSelection?.();
  if (selection && !selection.isCollapsed && selection.toString().length > 0) return;
  toggle();
}

function summaryKey(event: KeyboardEvent, toggle: () => void): void {
  if (event.target !== event.currentTarget || event.altKey || event.ctrlKey || event.metaKey) return;
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    toggle();
  }
}

export function ReaderNoticeBar(props: ReaderNoticeBarProps): JSX.Element {
  if(props.embedded)return <div class={css.segment} role="group" aria-label={props.label} data-testid={props.testid} data-state={props.state} data-tone={props.tone??'neutral'}><span class={css.mark} aria-hidden="true"><Icon name={props.icon}/></span><span class={css.summary} role={props.live?'status':undefined} aria-live={props.live?'polite':undefined} data-tooltip={props.tooltip} tabIndex={0}>{props.children}</span><Show when={props.actions}><span class={css.actions}>{props.actions}</span></Show>{props.after}</div>;
  return (
    <section
      class={css.root}
      aria-label={props.label}
      data-tooltip-placement="below"
      data-testid={props.testid}
      data-notice-bar=""
      data-state={props.state}
      data-tone={props.tone ?? 'neutral'}
    >
      <div
        class={props.onToggle ? `${css.row} ${css.rowToggle}` : css.row}
        data-toggle-row={props.onToggle ? '' : undefined}
        onClick={props.onToggle ? (event) => rowClick(event, props.onToggle!) : undefined}
      >
        <span class={css.mark} aria-hidden="true"><Icon name={props.icon} /></span>
        <span
          class={css.summary}
          role={props.live ? 'status' : undefined}
          aria-live={props.live ? 'polite' : undefined}
          data-tooltip={props.tooltip}
          tabIndex={0}
          onKeyDown={props.onToggle ? (event) => summaryKey(event, props.onToggle!) : undefined}
        >
          {props.children}
        </span>
        <Show when={props.actions}><span class={css.actions}>{props.actions}</span></Show>
      </div>
      {props.after}
    </section>
  );
}

export interface NoticeButtonProps {
  icon: IconName;
  /** Visible label (hidden in a narrow container) and default accessible name. */
  label: string;
  ariaLabel?: string;
  tooltip?: string;
  /** Primary actions retain their visible label in narrow containers. */
  primary?: boolean;
  /** An icon-only button: the chevron of a menu, or any action with no label. */
  iconOnly?: boolean;
  /** Draw the icon as the menu chevron (pointing down). */
  menu?: boolean;
  disabled?: boolean;
  busy?: boolean;
  /** Show the shared Spinner in place of the icon (work in progress). */
  spinner?: boolean;
  expanded?: boolean;
  haspopup?: 'menu' | 'dialog';
  controls?: string;
  testid?: string;
  /** Kept local in a detached reader (the remote-images grant buttons). */
  detachedLocal?: boolean;
  onClick: (event: MouseEvent) => void;
  onKeyDown?: (event: KeyboardEvent) => void;
}

/** An action in a notice bar: the remote-images banner's button style. */
export function NoticeButton(props: NoticeButtonProps): JSX.Element {
  const icon = (): JSX.Element => (props.spinner ? <Spinner /> : props.menu ? <span class={css.chevron}><Icon name={props.icon} /></span> : <Icon name={props.icon} />);
  return (
    <button
      type="button"
      class={`${css.btn}${props.primary ? ` ${css.btnPrimary}` : ''}${props.iconOnly ? ` ${css.iconBtn}` : ''}`}
      disabled={props.disabled}
      aria-busy={props.busy ? true : undefined}
      aria-expanded={props.expanded}
      aria-haspopup={props.haspopup}
      aria-controls={props.controls}
      aria-label={props.ariaLabel ?? props.label}
      data-tooltip={props.tooltip ?? props.label}
      data-testid={props.testid}
      data-detached-local={props.detachedLocal ? '' : undefined}
      onClick={(e) => props.onClick(e)}
      onKeyDown={(e) => props.onKeyDown?.(e)}
    >
      {icon()}
      <Show when={!props.iconOnly}><span class={css.label}>{props.label}</span></Show>
    </button>
  );
}
