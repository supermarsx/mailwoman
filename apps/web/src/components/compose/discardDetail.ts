import { t } from '../../i18n/index.ts';

/** The discard confirmation's sentence; a draft without a subject is named "this draft", not by the placeholder. */
export function discardDetail(subject: string | undefined): string {
  const named = (subject ?? '').trim();
  return named === '' ? t('mail-draft-discard-confirm-detail-untitled') : t('mail-draft-discard-confirm-detail', { subject: named });
}
