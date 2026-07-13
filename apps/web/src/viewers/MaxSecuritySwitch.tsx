// The three-position max-security opening switch (plan §3 e5, §7.2).
//
// A themed segmented control mounted into the Reader toolbar (e8 wires `value`/
// `onChange` to the `createMaxSecurityStore` in max-security.ts). Presentational
// only: it renders the current effective mode and reports a chosen one. When an
// admin floor is in force, positions LESS locked-down than the floor are
// disabled — the switch can never drop below the configured minimum.
//
// A11y: an ARIA radiogroup with roving tabindex. Left/Right (and Up/Down) move
// between enabled positions and select; Home/End jump to the first/last enabled.

import { For, type JSX } from 'solid-js';
import {
  SECURITY_MODES,
  SECURITY_MODE_LABELS,
  SECURITY_MODE_HINTS,
  isAtLeastAsStrict,
  type SecurityMode,
} from './max-security.ts';
import './max-security-switch.css';

export interface MaxSecuritySwitchProps {
  /** Currently selected mode (the effective mode for the open message). */
  value: SecurityMode;
  /** Called with the chosen mode; ignored for disabled (below-floor) positions. */
  onChange: (mode: SecurityMode) => void;
  /** Admin floor — positions below it are disabled. `null`/omitted = none. */
  floor?: SecurityMode | null;
  /** Accessible label for the group (defaults to a generic one). */
  label?: string;
}

/** Is a position selectable given the floor? */
function enabled(mode: SecurityMode, floor: SecurityMode | null | undefined): boolean {
  return floor === null || floor === undefined || isAtLeastAsStrict(mode, floor);
}

export function MaxSecuritySwitch(props: MaxSecuritySwitchProps): JSX.Element {
  const selectableModes = (): SecurityMode[] =>
    SECURITY_MODES.filter((m) => enabled(m, props.floor));

  function choose(mode: SecurityMode): void {
    if (enabled(mode, props.floor)) props.onChange(mode);
  }

  function onKeyDown(e: KeyboardEvent): void {
    const modes = selectableModes();
    if (modes.length === 0) return;
    const idx = Math.max(0, modes.indexOf(props.value));
    let next: SecurityMode | undefined;
    switch (e.key) {
      case 'ArrowRight':
      case 'ArrowDown':
        next = modes[(idx + 1) % modes.length];
        break;
      case 'ArrowLeft':
      case 'ArrowUp':
        next = modes[(idx - 1 + modes.length) % modes.length];
        break;
      case 'Home':
        next = modes[0];
        break;
      case 'End':
        next = modes[modes.length - 1];
        break;
      default:
        return;
    }
    if (next !== undefined) {
      choose(next);
      e.preventDefault();
    }
  }

  return (
    <div
      class="mw-maxsec"
      role="radiogroup"
      aria-label={props.label ?? 'Message security level'}
      data-testid="max-security-switch"
      data-mode={props.value}
      onKeyDown={onKeyDown}
    >
      <For each={SECURITY_MODES}>
        {(mode) => {
          const isSelected = (): boolean => props.value === mode;
          const isEnabled = (): boolean => enabled(mode, props.floor);
          return (
            <button
              type="button"
              role="radio"
              class="mw-maxsec__opt"
              classList={{
                'mw-maxsec__opt--on': isSelected(),
                'mw-maxsec__opt--off': !isEnabled(),
              }}
              aria-checked={isSelected()}
              aria-disabled={!isEnabled()}
              disabled={!isEnabled()}
              tabindex={isSelected() ? 0 : -1}
              title={SECURITY_MODE_HINTS[mode]}
              data-mode={mode}
              data-testid={`max-security-opt-${mode}`}
              onClick={() => choose(mode)}
            >
              {SECURITY_MODE_LABELS[mode]}
            </button>
          );
        }}
      </For>
    </div>
  );
}

export default MaxSecuritySwitch;
