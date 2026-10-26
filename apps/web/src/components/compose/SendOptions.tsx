// Per-message send options: read receipts, delivery notifications, and open tracking.
//
// Receipt defaults come from account settings; a stored draft keeps its own
// choices. Each option explains its scope beside the checkbox.

import { Show, type JSX } from 'solid-js';
import { t } from '../../i18n/index.ts';
import './compose-extras.css';

export function SendOptions(props: {
  /** The server can ask for a receipt (the engine builds the header). */
  receiptAvailable: boolean;
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
  return (
    <Show when={props.receiptAvailable || props.trackingAvailable || props.deliveryAvailable !== undefined}>
      <fieldset class="compose-options" data-testid="compose-send-options">
        <legend class="compose-options__legend">{t('mail-compose-options')}</legend>
        <Show when={props.receiptAvailable}>
          <label class="compose-options__toggle">
            <input
              type="checkbox"
              checked={props.requestReceipt}
              data-testid="compose-request-receipt"
              aria-describedby="compose-receipt-hint"
              onChange={(e) => props.onRequestReceipt(e.currentTarget.checked)}
            />
            <span>{t('mail-compose-request-receipt')}</span>
          </label>
          <p id="compose-receipt-hint" class="compose-options__hint">
            {t('mail-compose-request-receipt-hint')}
          </p>
        </Show>
        <Show when={props.deliveryAvailable !== undefined}>
          <label class="compose-options__toggle"><input type="checkbox" checked={props.requestDelivery === true}
            disabled={!props.deliveryAvailable} data-testid="compose-request-delivery" aria-describedby="compose-delivery-hint"
            onChange={event=>props.onRequestDelivery?.(event.currentTarget.checked)}/><span>{t('mail-compose-request-delivery')}</span></label>
          <p id="compose-delivery-hint" class="compose-options__hint">{t(props.deliveryAvailable?'mail-compose-request-delivery-hint':'mail-compose-delivery-unavailable')}</p>
        </Show>
        <Show when={props.trackingAvailable}>
          <label class="compose-options__toggle">
            <input
              type="checkbox"
              checked={props.tracking}
              data-testid="compose-track-opens"
              aria-describedby="compose-tracking-hint"
              onChange={(e) => props.onTracking(e.currentTarget.checked)}
            />
            <span>{t('mail-compose-track-opens')}</span>
          </label>
          <p id="compose-tracking-hint" class="compose-options__hint">
            {t('mail-compose-track-opens-hint')}
          </p>
        </Show>
      </fieldset>
    </Show>
  );
}
