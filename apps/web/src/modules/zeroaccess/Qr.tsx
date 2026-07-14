// Inline-SVG QR renderer (plan §3 e8). Paints the boolean module matrix from `qr.ts`
// as crisp black squares on white — self-contained, no external QR dependency, no
// network. Used to show the new device's ephemeral pairing public point (§9.1).

import { createMemo, For, type JSX } from 'solid-js';
import { encodeQr, type EcLevel } from './qr.ts';
import { t } from '../../i18n';
import * as css from './styles.css.ts';

export interface QrProps {
  /** The text to encode (here: the base64 ephemeral pairing public point). */
  value: string;
  ecLevel?: EcLevel;
  /** Accessible label for the QR image. */
  label?: string;
}

export function Qr(props: QrProps): JSX.Element {
  const matrix = createMemo(() => encodeQr(props.value, props.ecLevel ?? 'M'));
  const size = createMemo(() => matrix().length);
  const quiet = 4; // standard quiet zone
  const dim = createMemo(() => size() + quiet * 2);

  return (
    <svg
      class={css.qrFrame}
      viewBox={`0 0 ${dim()} ${dim()}`}
      role="img"
      aria-label={props.label ?? t('security-pair-qr-default')}
      data-testid="pairing-qr"
      shape-rendering="crispEdges"
    >
      <rect x="0" y="0" width={dim()} height={dim()} fill="#ffffff" />
      <For each={matrix()}>
        {(rowCells, r) => (
          <For each={rowCells}>
            {(on, c) =>
              on ? <rect x={c() + quiet} y={r() + quiet} width="1" height="1" fill="#000000" /> : null
            }
          </For>
        )}
      </For>
    </svg>
  );
}
