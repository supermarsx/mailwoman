import { onCleanup, type JSX } from 'solid-js';
import * as css from './ResizeHandle.css.ts';

export interface ResizeHandleProps {
  label: string;
  controls?: string;
  /** The separator's orientation: vertical changes width, horizontal changes height. */
  orientation?: 'vertical' | 'horizontal';
  value: number;
  min: number;
  max: number;
  /** Use -1 when the pane is after the separator, or for an RTL leading pane. */
  direction?: 1 | -1;
  onChange: (value: number) => void;
  onCommit?: (value: number) => void;
}

/** A controlled splitter. Persist onCommit; onChange supplies immediate drag feedback. */
export function ResizeHandle(props: ResizeHandleProps): JSX.Element {
  const horizontal = () => props.orientation === 'horizontal';
  const clamp = (value: number) => Math.round(Math.max(props.min, Math.min(props.max, value)));
  let drag: { id: number; origin: number; initial: number; current: number; element: HTMLDivElement } | undefined;
  const coordinate = (event: PointerEvent) => horizontal() ? event.clientY : event.clientX;
  const finish = (commit: boolean) => {
    const active = drag;
    if (!active) return;
    drag = undefined;
    if (active.element.hasPointerCapture?.(active.id)) active.element.releasePointerCapture(active.id);
    if (commit) props.onCommit?.(active.current);
    else props.onChange(clamp(active.initial));
  };
  onCleanup(() => { drag = undefined; });
  return <div
    class={css.handle}
    role="separator"
    tabIndex={0}
    aria-label={props.label}
    aria-controls={props.controls}
    aria-orientation={props.orientation ?? 'vertical'}
    aria-valuemin={props.min}
    aria-valuemax={props.max}
    aria-valuenow={clamp(props.value)}
    data-orientation={props.orientation ?? 'vertical'}
    onPointerDown={(event) => {
      if (event.button !== 0 || drag) return;
      event.preventDefault();
      event.currentTarget.focus();
      event.currentTarget.setPointerCapture(event.pointerId);
      drag = { id: event.pointerId, origin: coordinate(event), initial: props.value, current: clamp(props.value), element: event.currentTarget };
    }}
    onPointerMove={(event) => {
      if (!drag || event.pointerId !== drag.id) return;
      drag.current = clamp(drag.initial + (coordinate(event) - drag.origin) * (props.direction ?? 1));
      props.onChange(drag.current);
    }}
    onPointerUp={(event) => { if (drag?.id === event.pointerId) finish(true); }}
    onPointerCancel={(event) => { if (drag?.id === event.pointerId) finish(false); }}
    onLostPointerCapture={() => finish(true)}
    onKeyDown={(event) => {
      if (event.key === 'Escape' && drag) { event.preventDefault(); finish(false); return; }
      const decrease = horizontal() ? 'ArrowUp' : 'ArrowLeft';
      const increase = horizontal() ? 'ArrowDown' : 'ArrowRight';
      const step = event.shiftKey ? 40 : 10;
      let next: number;
      if (event.key === 'Home') next = props.min;
      else if (event.key === 'End') next = props.max;
      else if (event.key === decrease || event.key === increase) {
        next = props.value + (event.key === increase ? step : -step) * (props.direction ?? 1);
      } else return;
      event.preventDefault();
      next = clamp(next);
      props.onChange(next);
      props.onCommit?.(next);
    }}
  />;
}
