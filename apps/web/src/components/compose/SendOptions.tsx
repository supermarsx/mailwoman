// Per-message send options: read receipts, delivery notifications, and open tracking.
// Rendered as rows of the composer ribbon's Delivery group (t38-e5).
//
// Receipt defaults come from account settings; a stored draft keeps its own
// choices. Each option explains its scope beside the checkbox.

import { Show, type JSX } from 'solid-js';
import { RibbonItem } from '../bars/index.ts';
import { t } from '../../i18n/index.ts';
import './compose-extras.css';

export function SendOptions(props: {
  /** The server can ask for a receipt (the engine builds the header). */
  receiptAvailable: boolean;
  receiptLocked?:boolean;
  deliveryLocked?:boolean;
  deliveryAvailable?: boolean;
  requestDelivery?: boolean;
  onRequestDelivery?: (on:boolean)=>void;
  requestReceipt: boolean;
  onRequestReceipt: (on: boolean) => void;
  /** Open tracking is on at this server and the message is neither encrypted
   *  nor signed. When false the option is not rendered at all. */
  trackingAvailable: boolean;
  tracking: boolean;
  onTracking: (on: boolean) => void;
}): JSX.Element {
  const deliveryHint = (): string =>
    t(props.deliveryAvailable ? 'mail-compose-request-delivery-hint' : 'mail-compose-delivery-unavailable');
  // One ribbon row per option (t38): the Delivery group's 3-row grid. Each
  // hint stays the option's description, read out and shown as its tooltip.
  return (
    <>
      <Show when={props.receiptAvailable}>
        <RibbonItem><div class="compose-rib__row">
          <label class="compose-options__toggle" data-tooltip={t('mail-compose-request-receipt-hint')}>
            <input
              type="checkbox"
              checked={props.requestReceipt}
              disabled={props.receiptLocked}
              data-testid="compose-request-receipt"
              aria-describedby="compose-receipt-hint"
              onChange={(e) => props.onRequestReceipt(e.currentTarget.checked)}
            />
            <span class="compose-rib__text">{t('mail-compose-request-receipt')}</span>
          </label>
          <span id="compose-receipt-hint" hidden>
            {t('mail-compose-request-receipt-hint')}
          </span>
        </div></RibbonItem>
      </Show>
      <Show when={props.deliveryAvailable !== undefined}>
        <RibbonItem><div class="compose-rib__row">
          <label class="compose-options__toggle" data-tooltip={deliveryHint()}>
            <input type="checkbox" checked={props.requestDelivery === true}
              disabled={!props.deliveryAvailable||props.deliveryLocked} data-testid="compose-request-delivery" aria-describedby="compose-delivery-hint"
              onChange={event=>props.onRequestDelivery?.(event.currentTarget.checked)}/>
            <span class="compose-rib__text">{t('mail-compose-request-delivery')}</span>
          </label>
          <span id="compose-delivery-hint" hidden>{deliveryHint()}</span>
        </div></RibbonItem>
      </Show>
      <Show when={props.trackingAvailable}>
        <RibbonItem><div class="compose-rib__row">
          <label class="compose-options__toggle" data-tooltip={t('mail-compose-track-opens-hint')}>
            <input
              type="checkbox"
              checked={props.tracking}
              data-testid="compose-track-opens"
              aria-describedby="compose-tracking-hint"
              onChange={(e) => props.onTracking(e.currentTarget.checked)}
            />
            <span class="compose-rib__text">{t('mail-compose-track-opens')}</span>
          </label>
          <span id="compose-tracking-hint" hidden>
            {t('mail-compose-track-opens-hint')}
          </span>
        </div></RibbonItem>
      </Show>
    </>
  );
}
