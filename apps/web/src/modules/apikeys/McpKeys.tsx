// MCP-key management (SPEC §20.3, plan §2.4 / §3 e8). MCP keys ARE API keys (same
// scoping/expiry/rate-limit/audit) with per-tool grants (`mcp_tools`). This builds an
// MCP-scoped `ApiKeyScope`: each tool is individually grantable, and enabling the
// `mail.send` tool surfaces the SAFETY-CRITICAL `unattended_send` disclosure (default
// off → sends land in the Outbox for in-app confirmation; on → requires an admin
// countersign, see §2.4 / R4).
//
// EXPORTED for e11 to mount; does not touch the router or Settings.tsx.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { ApiKeyService, type Fetcher } from './service.ts';
import { MCP_TOOLS, readOnlyScope, type ApiKeyScope, type MintedKey } from './types.ts';
import * as css from './styles.css.ts';

/** Fluent message id for an MCP tool's `field` (`label`/`desc`); ids use `-`. */
const toolMessageId = (toolId: string, field: 'label' | 'desc'): string =>
  `apikeys-tool-${toolId.replace(/\./g, '-')}-${field}`;

export interface McpKeysProps {
  accountId: string;
  fetcher?: Fetcher;
}

function withTool(scope: ApiKeyScope, toolId: string, on: boolean, sends: boolean): ApiKeyScope {
  const set = new Set(scope.mcpTools);
  if (on) set.add(toolId);
  else set.delete(toolId);
  const mcpTools = [...set];
  // Enabling the send tool grants the `send` verb; removing it drops unattended send.
  const send = sends ? on || scope.send : scope.send;
  const unattendedSend = mcpTools.includes('mail.send') ? scope.unattendedSend : false;
  return { ...scope, mcpTools, send, unattendedSend };
}

export function McpKeys(props: McpKeysProps): JSX.Element {
  onMount(() => void loadCatalog('apikeys'));
  const service = new ApiKeyService(props.fetcher);
  const [label, setLabel] = createSignal('');
  const [copied, setCopied] = createSignal(false);

  async function copySecret(secret: string): Promise<void> {
    try {
      await navigator.clipboard?.writeText(secret);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  }

  const [scope, setScope] = createSignal<ApiKeyScope>({ ...readOnlyScope(props.accountId), mcpTools: [] });
  const [minted, setMinted] = createSignal<MintedKey | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  const sendGranted = (): boolean => scope().mcpTools.includes('mail.send');

  async function onCreate(): Promise<void> {
    setError('');
    setMinted(null);
    setCopied(false);
    if (label().trim() === '') {
      setError(t('apikeys-mcp-error-need-label'));
      return;
    }
    if (scope().mcpTools.length === 0) {
      setError(t('apikeys-mcp-error-need-tool'));
      return;
    }
    setBusy(true);
    try {
      setMinted(await service.create({ label: label().trim(), accountId: props.accountId, scope: scope() }));
      setLabel('');
      setScope({ ...readOnlyScope(props.accountId), mcpTools: [] });
    } catch (e) {
      setError(e instanceof Error ? e.message : t('apikeys-mcp-error-create'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} aria-label={t('apikeys-mcp-panel-label')}>
      <section class={css.section}>
        <h2 class={css.heading}>{t('apikeys-mcp-heading')}</h2>
        <p class={css.prose}>{t('apikeys-mcp-intro')}</p>

        <label class={css.field}>
          <span class={css.subHeading}>{t('apikeys-label')}</span>
          <input
            class={css.input}
            value={label()}
            placeholder={t('apikeys-mcp-label-placeholder')}
            aria-label={t('apikeys-mcp-label-aria')}
            onInput={(e) => setLabel(e.currentTarget.value)}
          />
        </label>

        <span class={css.subHeading}>{t('apikeys-tools')}</span>
        <div class={css.grid} role="group" aria-label={t('apikeys-tools-group')}>
          <For each={MCP_TOOLS}>
            {(tool) => (
              <label class={css.check} title={t(toolMessageId(tool.id, 'desc'))}>
                <input
                  class={css.checkbox}
                  type="checkbox"
                  checked={scope().mcpTools.includes(tool.id)}
                  onChange={(e) => setScope(withTool(scope(), tool.id, e.currentTarget.checked, tool.sends))}
                />
                {t(toolMessageId(tool.id, 'label'))}
                {tool.sends ? ` ${t('apikeys-outbox-suffix')}` : ''}
              </label>
            )}
          </For>
        </div>

        <Show when={sendGranted()}>
          <div class={css.field} data-testid="unattended-send-block">
            <p class={css.warn} data-testid="unattended-send-disclosure">
              {t('apikeys-unattended-disclosure')}
            </p>
            <label class={css.check}>
              <input
                class={css.checkbox}
                type="checkbox"
                checked={scope().unattendedSend}
                aria-label={t('apikeys-unattended-aria')}
                onChange={(e) => setScope({ ...scope(), unattendedSend: e.currentTarget.checked })}
              />
              {t('apikeys-unattended-allow')}
            </label>
          </div>
        </Show>

        <button type="button" class={css.button} disabled={busy()} onClick={() => void onCreate()}>
          {t('apikeys-mcp-create')}
        </button>

        <Show when={minted()}>
          {(m) => (
            <div class={css.field} data-testid="minted-mcp-key">
              <p class={css.warn}>{t('apikeys-mcp-reveal-warning')}</p>
              <code class={css.token} data-testid="minted-mcp-token">
                {m().displayToken}
              </code>
              <Show when={scope().unattendedSend || m().record.scope.unattendedSend}>
                <p class={css.prose}>{t('apikeys-unattended-pending')}</p>
              </Show>
              <div class={css.row}>
                <button
                  type="button"
                  class={css.ghost}
                  aria-label={t('apikeys-copy-aria')}
                  onClick={() => void copySecret(m().displayToken)}
                >
                  {t('apikeys-copy')}
                </button>
                <button type="button" class={css.ghost} onClick={() => setMinted(null)}>
                  {t('apikeys-saved')}
                </button>
              </div>
              <p class={css.copiedNote} role="status" aria-live="polite">
                {copied() ? t('apikeys-copied') : ''}
              </p>
            </div>
          )}
        </Show>

        <Show when={error() !== ''}>
          <p class={css.error} role="alert">
            {error()}
          </p>
        </Show>
      </section>
    </div>
  );
}

export default McpKeys;

/** Exported for tests verifying the grant→verb/unattended coupling. */
export { withTool };
