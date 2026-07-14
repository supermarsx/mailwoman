// MCP-key management (SPEC §20.3, plan §2.4 / §3 e8). MCP keys ARE API keys (same
// scoping/expiry/rate-limit/audit) with per-tool grants (`mcp_tools`). This builds an
// MCP-scoped `ApiKeyScope`: each tool is individually grantable, and enabling the
// `mail.send` tool surfaces the SAFETY-CRITICAL `unattended_send` disclosure (default
// off → sends land in the Outbox for in-app confirmation; on → requires an admin
// countersign, see §2.4 / R4).
//
// EXPORTED for e11 to mount; does not touch the router or Settings.tsx.

import { createSignal, For, Show, type JSX } from 'solid-js';
import { ApiKeyService, type Fetcher } from './service.ts';
import {
  MCP_TOOLS,
  UNATTENDED_SEND_DISCLOSURE,
  readOnlyScope,
  type ApiKeyScope,
  type MintedKey,
} from './types.ts';
import * as css from './styles.css.ts';

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
  const service = new ApiKeyService(props.fetcher);
  const [label, setLabel] = createSignal('');
  const [scope, setScope] = createSignal<ApiKeyScope>({ ...readOnlyScope(props.accountId), mcpTools: [] });
  const [minted, setMinted] = createSignal<MintedKey | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  const sendGranted = (): boolean => scope().mcpTools.includes('mail.send');

  async function onCreate(): Promise<void> {
    setError('');
    setMinted(null);
    if (label().trim() === '') {
      setError('give the MCP key a label');
      return;
    }
    if (scope().mcpTools.length === 0) {
      setError('grant at least one tool');
      return;
    }
    setBusy(true);
    try {
      setMinted(await service.create({ label: label().trim(), accountId: props.accountId, scope: scope() }));
      setLabel('');
      setScope({ ...readOnlyScope(props.accountId), mcpTools: [] });
    } catch (e) {
      setError(e instanceof Error ? e.message : 'could not create the MCP key');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} aria-label="MCP keys">
      <section class={css.section}>
        <h2 class={css.heading}>MCP keys</h2>
        <p class={css.prose}>
          Grant an AI agent (over the Model Context Protocol) exactly the tools you choose. Mail
          bodies returned to the agent are labelled untrusted input. Each tool is granted
          individually.
        </p>

        <label class={css.field}>
          <span class={css.subHeading}>Label</span>
          <input
            class={css.input}
            value={label()}
            placeholder="e.g. assistant agent"
            aria-label="MCP key label"
            onInput={(e) => setLabel(e.currentTarget.value)}
          />
        </label>

        <span class={css.subHeading}>Tools</span>
        <div class={css.grid} role="group" aria-label="MCP tools">
          <For each={MCP_TOOLS}>
            {(tool) => (
              <label class={css.check} title={tool.description}>
                <input
                  type="checkbox"
                  checked={scope().mcpTools.includes(tool.id)}
                  onChange={(e) => setScope(withTool(scope(), tool.id, e.currentTarget.checked, tool.sends))}
                />
                {tool.label}
                {tool.sends ? ' (Outbox-gated)' : ''}
              </label>
            )}
          </For>
        </div>

        <Show when={sendGranted()}>
          <div class={css.field} data-testid="unattended-send-block">
            <p class={css.warn} data-testid="unattended-send-disclosure">
              {UNATTENDED_SEND_DISCLOSURE}
            </p>
            <label class={css.check}>
              <input
                type="checkbox"
                checked={scope().unattendedSend}
                aria-label="Unattended send"
                onChange={(e) => setScope({ ...scope(), unattendedSend: e.currentTarget.checked })}
              />
              Allow unattended send (bypass the Outbox — requires admin countersign)
            </label>
          </div>
        </Show>

        <button type="button" class={css.button} disabled={busy()} onClick={() => void onCreate()}>
          Create MCP key
        </button>

        <Show when={minted()}>
          {(m) => (
            <div class={css.field} data-testid="minted-mcp-key">
              <p class={css.warn}>Copy this secret now — it is shown once.</p>
              <code class={css.token} data-testid="minted-mcp-token">
                {m().displayToken}
              </code>
              <Show when={scope().unattendedSend || m().record.scope.unattendedSend}>
                <p class={css.prose}>
                  Unattended send stays inactive until an administrator countersigns this key.
                </p>
              </Show>
              <button type="button" class={css.ghost} onClick={() => setMinted(null)}>
                I have saved it
              </button>
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
