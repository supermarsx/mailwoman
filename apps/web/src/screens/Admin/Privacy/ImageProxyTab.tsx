import { Show, For, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { AdminImageConcurrency, IMAGE_CONCURRENCY_KEY } from '../AdminImageConcurrency.tsx';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { ImageProxyReadout } from '../Settings/ImageProxyReadout.tsx';
import { InlineGroup, SettingsGate } from './InlineGroup.tsx';
import { imageGroup, imageProxyRows, type ImageGroup, type PrivacyTabContext } from './model.ts';
import { HistoryControls, ProxyCharts } from './ProxyCharts.tsx';

const GROUPS: readonly ImageGroup[] = ['rate', 'cache', 'ram', 'disk', 'tiering'];

export default function ImageProxyTab(props: { ctx: PrivacyTabContext; canWrite: boolean }): JSX.Element {
  const ctx = (): PrivacyTabContext => props.ctx;
  const mode = () => ctx().rows().find((row) => row.key === IMAGE_CONCURRENCY_KEY);
  const stats = () => (ctx().statsError() ? undefined : ctx().stats());
  return (
    <>
      <AdminSubsection title={t('privacy-history-worker-title')}>
        <HistoryControls range={ctx().range()} onRange={ctx().setRange} updatedAt={ctx().updatedAt()} loading={ctx().statsLoading()} onRefresh={ctx().refresh} history={stats()?.history} />
        <ImageProxyReadout shared={stats} />
        <ProxyCharts stats={stats()} loading={ctx().statsLoading()} error={ctx().statsError()} view="proxy" />
      </AdminSubsection>
      <Show when={mode() || stats()?.imageConcurrency}>
        <AdminSubsection title={t('privacy-concurrency-title')}>
          <AdminImageConcurrency
            setting={mode()}
            runtime={stats()?.imageConcurrency}
            rate={stats()?.imageRate}
            cache={stats()?.imageCache}
            canWrite={props.canWrite}
            {...(ctx().saveMode ? { save: ctx().saveMode! } : {})}
            onSaved={ctx().onSaved}
            onReload={ctx().reloadSettings}
          />
        </AdminSubsection>
      </Show>
      <SettingsGate ctx={ctx()}>
        <For each={GROUPS}>{(group) => (
          <InlineGroup ctx={ctx()} title={t(`privacy-image-group-${group}`)} rows={imageProxyRows(ctx().rows()).filter((row) => imageGroup(row.key) === group)} />
        )}</For>
      </SettingsGate>
    </>
  );
}
