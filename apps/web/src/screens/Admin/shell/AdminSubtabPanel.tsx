import { Suspense, type JSX } from 'solid-js';
import { InlineLoading } from '../../../components/InlineLoading.tsx';
import { t } from '../../../i18n';

/** Keep section navigation mounted while a subtab loads its own code or data. */
export function AdminSubtabPanel(props: { children: JSX.Element }): JSX.Element {
  return <Suspense fallback={<InlineLoading message={t('admin-shell-loading')} />}>{props.children}</Suspense>;
}
