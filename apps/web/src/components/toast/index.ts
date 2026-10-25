// The shared toast system (t32 f39). Import from here:
//   import { toast } from '<rel>/components/toast/index.ts';
//   toast.success(t('…-saved'));  toast.error(msg, { action: { label, run } });
// and mount <ToastRegion /> once per app shell.
export {
  toast,
  toasts,
  setToastPause,
  toastsPaused,
  toastTimeLeft,
  resetToasts,
  TOAST_DURATIONS,
  TOAST_LIMIT,
  type ToastAction,
  type ToastCloseReason,
  type ToastItem,
  type ToastOptions,
  type ToastTone,
  type PauseReason,
} from './toast.ts';
export { ToastRegion } from './ToastRegion.tsx';
