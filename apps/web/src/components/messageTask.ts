import type { Email } from '../api/jmap-types.ts';
import { preferenceAccount, preferenceAccountIsCurrent } from '../api/accountPreferenceLifecycle.ts';
import { featureOn } from '../state/features.ts';
import type { useApp } from '../state/context.ts';
import { loadCatalog, t } from '../i18n';

type App = ReturnType<typeof useApp>;
const pending = new Set<string>();
void loadCatalog('message-task');

export async function createMessageTask(app: App, email: Email): Promise<void> {
  const owner = preferenceAccount();
  if (!owner || owner.id !== app.accountId() || !featureOn('tasks')) return;
  const key = JSON.stringify([owner.generation, email.id]);
  if (pending.has(key)) return;
  pending.add(key);
  const current = () => preferenceAccountIsCurrent(owner) && app.accountId() === owner.id && featureOn('tasks');
  try {
    if (!app.taskLists().length) await app.loadTasks();
    if (!current()) return;
    const list = app.taskLists().find(row => row.id === app.selectedListId()) ?? app.taskLists()[0];
    if (!list) {
      app.showToast('error', t('message-task-no-list'));
      return;
    }
    const id = await app.convertMailToTask(email.id, list.id, { title: email.subject?.trim() || t('mail-no-subject') });
    if (!current()) return;
    if (id) app.showToast('success', t('message-task-created', { list: list.name }));
    else app.showToast('error', t('message-task-failed'));
  } catch {
    if (current()) app.showToast('error', t('message-task-failed'));
  } finally {
    pending.delete(key);
  }
}
