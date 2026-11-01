import { Suspense, type JSX } from 'solid-js';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import { t } from '../../../i18n';

/** Keep section navigation mounted while a subtab loads its own code or data. */
export function AdminSubtabPanel(props: { children: JSX.Element }): JSX.Element {
  return <Suspense fallback={<AsyncPending variant="lines" message={t('admin-shell-loading')} />}>{props.children}</Suspense>;
}
