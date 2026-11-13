import { createMemo, Show, type JSX } from 'solid-js';
import { settings } from '../api/settings.ts';
import { TemplateLibraryEditor } from '../components/compose/TemplateLibraryEditor.tsx';
import { validResponseTemplates, type ResponseTemplate } from '../components/compose/responseTemplateModel.ts';
import type { ComposeInitial } from '../components/compose/reply.ts';
import { loadCatalog, t } from '../i18n';
import { escapeHtml } from '../viewers/sandbox.ts';
import { ResponseTemplatePreferences } from './Settings/ResponseTemplatePreferences.tsx';
import { SettingsCard } from './Settings/layout/SettingsCard.tsx';
import * as table from './Settings/managementTable.css.ts';

export default function Templates(props: { onCompose: (draft: ComposeInitial) => void }): JSX.Element {
  void loadCatalog('response-templates');
  const enabled = () => settings()['mail.templates.enabled']?.value === true;
  const shared = createMemo(() => {
    const value = settings()['mail.templates.shared']?.value;
    return validResponseTemplates(value) ? value : [];
  });
  function compose(row: ResponseTemplate): void {
    if (!enabled() || !validResponseTemplates([row])) return;
    props.onCompose({
      mode: 'new',
      to: row.to ?? '',
      cc: row.cc ?? '',
      ...(row.bcc ? { bcc: row.bcc } : {}),
      subject: row.subject ?? '',
      bodyText: row.body,
      bodyHtml: `<p>${escapeHtml(row.body).replace(/\r\n?|\n/g, '<br>')}</p>`,
      inReplyTo: [],
      references: [],
      attachments: [],
      quotesDecrypted: false,
    });
  }

  return (
    <main class={`module-pane ${table.workspace}`} data-surface="templates" data-mw-surface>
      <h2>{t('response-templates-workspace')}</h2>
      <p>{t('response-templates-workspace-help')}</p>
      <Show when={enabled()}>
        <SettingsCard id="shared-template-library" title={t('response-templates-shared')} wide>
          <TemplateLibraryEditor value={shared()} onChange={() => {}} disabled onUse={compose} />
        </SettingsCard>
        <ResponseTemplatePreferences onUse={compose} />
      </Show>
    </main>
  );
}
