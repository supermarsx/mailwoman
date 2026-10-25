// Per-message send options (t29-e12): ask for a read receipt, record opens.
//
// Both start off for every message and are not remembered for the next one (a
// stored draft keeps them for that draft only). Each option says what it does
// and what it cannot do, next to the checkbox.

import { Show, type JSX } from 'solid-js';
import { t } from '../../i18n/index.ts';
import './compose-extras.css';

export function SendOptions(props: {
  /** The server can ask for a receipt (the engine builds the header). */
  receiptAvailable: boolean;
  requestReceipt: boolean;
  onRequestReceipt: (on: boolean) => void;
  /** Open tracking is on at this server and the message is neither encrypted
   *  nor signed. When false the option is not rendered at all. */
  trackingAvailable: boolean;
  tracking: boolean;
  onTracking: (on: boolean) => void;
}): JSX.Element {
  return (
    <Show when={props.receiptAvailable || props.trackingAvailable}>
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
