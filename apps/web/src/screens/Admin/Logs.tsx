import { Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { AuthLogs } from './AuthLogs.tsx';
import { ApplicationLogs } from './ApplicationLogs.tsx';
import { useShell } from './shell/context.ts';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { sectionPath } from './shell/router.ts';
import * as ux from './shell/ux.css.ts';
export function Logs(): JSX.Element {
    const shell = useShell();
    const application = (): boolean => shell.router.route().rest[0] === 'application';
    return <div class={ux.page}>
        <SectionTabs label={t('admin-logs-title')} selected={application() ? 'application' : 'authentication'}
            items={[
                { id: 'authentication', label: t('admin-auth-title'), path: sectionPath('logs') },
                { id: 'application', label: t('admin-app-logs-title'), path: sectionPath('logs', ['application']) },
            ]} navigate={path => { void shell.router.navigate(path); }} />
        <Show when={application()} fallback={<AuthLogs />}><ApplicationLogs /></Show>
    </div>;
}
