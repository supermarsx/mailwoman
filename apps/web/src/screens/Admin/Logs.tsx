import { Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { AuthLogs } from './AuthLogs.tsx';
import { ApplicationLogs } from './ApplicationLogs.tsx';
import { useShell } from './shell/context.ts';
import * as ux from './shell/ux.css.ts';
export function Logs(): JSX.Element {
    const shell = useShell();
    const application = (): boolean => shell.router.route().rest[0] === 'application';
    return <><nav class={ux.pageActions} aria-label={t('admin-logs-title')}><a class="btn" href="#/logs" aria-current={!application() ? 'page' : undefined}>{t('admin-auth-title')}</a><a class="btn" href="#/logs/application" aria-current={application() ? 'page' : undefined}>{t('admin-app-logs-title')}</a></nav><Show when={application()} fallback={<AuthLogs />}><ApplicationLogs /></Show></>;
}
