// jsdom has no PointerEvent; testing-library then dispatches a plain Event and
// drops clientX/pointerId. Tests of pointer-driven code install this first.
export function installPointerEvent(): void {
  if (typeof globalThis.PointerEvent === 'function' && 'pointerId' in globalThis.PointerEvent.prototype) return;
  class TestPointerEvent extends MouseEvent {
    pointerId: number;
    pointerType: string;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 1;
      this.pointerType = init.pointerType ?? 'mouse';
    }
  }
  (globalThis as { PointerEvent: unknown }).PointerEvent = TestPointerEvent;
}
