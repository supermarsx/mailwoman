import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { AdminSecurityPolicy, policyValues, policyWrite, type SecurityPolicyApi } from './index.tsx';
import { parseDlpRules, validateDlpRules, newDlpRule } from './DlpRulesEditor.tsx';
import type { SecurityPolicy } from '../../../api/admin/policy.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle, problem } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';

beforeEach(() => history.replaceState(null, '', '/admin#/security-policy'));

const RULE = {
  id: 'cards',
  name: 'Card numbers',
  enabled: true,
  priority: 10,
  action: 'block',
  message: '',
  conditions: {
    detectors: ['pan'],
    customRegex: null,
    dictionaries: [],
    attachmentTypes: [],
    maxAttachmentSize: null,
    recipientDomains: [],
    recipientDomainMode: null,
    classification: null,
  },
} as const;

function policy(over: Partial<SecurityPolicy> = {}): SecurityPolicy {
  return {
    minTls: '1.2',
    require2fa: false,
    argon2MCost: 19456,
    argon2TCost: 2,
    argon2PCost: 1,
    capturePolicy: 'off',
    maxSecurityFloor: null,
    dlpRulesJson: JSON.stringify([RULE]),
    envDlpRules: [],
    pinnedByEnv: [],
    ...over,
  };
}

function setup(p: SecurityPolicy = policy(), save?: SecurityPolicyApi['save']) {
  const api: SecurityPolicyApi = {
    load: vi.fn(async () => ({ kind: 'ok' as const, data: p })),
    save: vi.fn(save ?? (async (w) => ({ ...p, ...w }))),
  };
  const utils = renderInShell(() => <AdminSecurityPolicy api={api} />);
  return { ...utils, api };
}

const saveButton = () => screen.getByRole('button', { name: 'Save' });

describe('Security policy screen', () => {
  it('shows each field with help, a Learn more link into docs/admin/security-policy.md and the 30 s note', async () => {
    setup();
    const tls = await screen.findByRole('combobox', { name: /Lowest TLS version/ });
    expect(tls).toHaveValue('1.2');
    expect(tls).toHaveAccessibleDescription(/HTTPS listener/);
    const link = screen.getByRole('link', { name: 'Learn more about Lowest TLS version' });
    expect(link.getAttribute('href')).toMatch(/docs\/admin\/security-policy\.md#minTls$/);
    expect(within(screen.getByTestId('setting-minTls')).getByText('Applies within 30 seconds on every server process.')).toBeInTheDocument();
    expect(within(screen.getByTestId('setting-minTls')).getByTestId('source-badge')).toHaveTextContent('Default');
    expect(screen.getByRole('spinbutton', { name: /Argon2 memory/ })).toHaveValue(19456);
    expect(screen.getByRole('link', { name: 'History' }).getAttribute('href')).toContain('audit?target=security_policy');
  });

  it('a field pinned by the environment is disabled and names its variable', async () => {
    setup(policy({ minTls: '1.3', pinnedByEnv: ['minTls'] }));
    const tls = await screen.findByRole('combobox', { name: /Lowest TLS version/ });
    expect(tls).toBeDisabled();
    expect(stripIsolates(document.getElementById(tls.getAttribute('aria-describedby')!.split(' ')[1]!)?.textContent ?? '')).toMatch(/MW_ADMIN_MIN_TLS/);
    expect(within(screen.getByTestId('setting-minTls')).getByTestId('source-badge')).toHaveTextContent(/MW_ADMIN_MIN_TLS/);
  });

  it('checks the Argon2 bounds before any request and shows the error at the field', async () => {
    const { api } = setup();
    const mem = await screen.findByRole('spinbutton', { name: /Argon2 memory/ });
    fireEvent.input(mem, { target: { value: '1000' } });
    fireEvent.click(saveButton());
    expect(await screen.findByText(/^Enter 19,?456 or more\.$/)).toBeInTheDocument();
    expect(mem).toHaveAttribute('aria-invalid', 'true');
    expect(api.save).not.toHaveBeenCalled();
    // A single pass needs more memory: the cross-field rule.
    fireEvent.input(mem, { target: { value: '20000' } });
    fireEvent.input(screen.getByRole('spinbutton', { name: /Argon2 passes/ }), { target: { value: '1' } });
    fireEvent.click(saveButton());
    expect(await screen.findByText(/^With a single pass the memory must be at least 47,?104 KiB\.$/)).toBeInTheDocument();
    expect(api.save).not.toHaveBeenCalled();
  });

  it('saves all eight fields, with the rules as JSON, and shows the stored result', async () => {
    const { api } = setup();
    fireEvent.change(await screen.findByRole('combobox', { name: /Reading floor/ }), { target: { value: 'plain-text' } });
    fireEvent.click(screen.getByRole('checkbox', { name: /Require a second factor/ }));
    expect(screen.getByTestId('save-bar')).toHaveTextContent('2 unsaved changes');
    fireEvent.click(saveButton());
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(1));
    expect(api.save).toHaveBeenCalledWith({
      minTls: '1.2',
      require2fa: true,
      argon2MCost: 19456,
      argon2TCost: 2,
      argon2PCost: 1,
      capturePolicy: 'off',
      maxSecurityFloor: 'plain-text',
      dlpRulesJson: JSON.stringify([RULE]),
    });
    await waitFor(() => expect(screen.getByTestId('save-result')).toHaveTextContent(/Saved 2 changes/));
    expect(screen.getByTestId('save-bar')).toHaveTextContent('No unsaved changes');
  });

  it("puts the server's 409 refusal at the pinned field", async () => {
    setup(policy(), async () => {
      throw problem(409, { detail: 'minTls is set by MW_ADMIN_MIN_TLS and cannot be changed here', field: '/minTls', code: 'pinned' });
    });
    fireEvent.change(await screen.findByRole('combobox', { name: /Lowest TLS version/ }), { target: { value: '1.3' } });
    fireEvent.click(saveButton());
    expect(await screen.findByText('minTls is set by MW_ADMIN_MIN_TLS and cannot be changed here')).toBeInTheDocument();
  });

  it('DLP rules are edited as rules: add one, a missing name blocks saving, a filled one is sent', async () => {
    const { api } = setup();
    const editor = await screen.findByTestId('dlp-editor');
    expect(within(editor).getAllByTestId('dlp-rule')).toHaveLength(1);
    fireEvent.click(within(editor).getByRole('button', { name: 'Add a rule' }));
    const rules = within(editor).getAllByTestId('dlp-rule');
    expect(rules).toHaveLength(2);
    fireEvent.click(saveButton());
    expect(await screen.findByText('Rule 2 needs a name.')).toBeInTheDocument();
    expect(api.save).not.toHaveBeenCalled();
    const second = rules[1]!;
    fireEvent.input(within(second).getByRole('textbox', { name: 'Name' }), { target: { value: 'Contracts' } });
    fireEvent.click(within(second).getByRole('checkbox', { name: 'Bank account numbers (IBAN)' }));
    fireEvent.click(saveButton());
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(1));
    const sent = JSON.parse((api.save as ReturnType<typeof vi.fn>).mock.calls[0]![0].dlpRulesJson) as { name: string; conditions: { detectors: string[] } }[];
    expect(sent.map((r) => r.name)).toEqual(['Card numbers', 'Contracts']);
    expect(sent[1]!.conditions.detectors).toEqual(['iban']);
  });

  it('rules from MW_DLP_RULES are listed read-only and a stored rule with the same id says it is not applied', async () => {
    setup(policy({ envDlpRules: [{ ...RULE, name: 'Env cards' } as never] }));
    const env = await screen.findByTestId('dlp-env-rules');
    expect(env).toHaveTextContent('Env cards');
    expect(within(env).queryByRole('textbox')).toBeNull();
    expect(screen.getByText(/A rule from MW_DLP_RULES uses the id/)).toBeInTheDocument();
  });

  it('saves Portuguese detectors and a custom keyword phrase without splitting it', async () => {
    const { api } = setup();
    const editor = await screen.findByTestId('dlp-editor');
    const rule = within(editor).getByTestId('dlp-rule');
    for (const name of [/Portuguese tax number/, /Portuguese Social Security/, /Portuguese Citizen Card/, /Password and credential/]) {
      fireEvent.click(within(rule).getByRole('checkbox', { name }));
    }
    const keywords = within(rule).getByRole('textbox', { name: 'Custom keywords and phrases' });
    fireEvent.input(keywords, { target: { value: 'internal secret' } });
    fireEvent.keyDown(keywords, { key: 'Enter' });
    fireEvent.click(saveButton());
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(1));
    const sent = JSON.parse((api.save as ReturnType<typeof vi.fn>).mock.calls[0]![0].dlpRulesJson) as { conditions: { detectors: string[]; dictionaries: string[] } }[];
    expect(sent[0]!.conditions.detectors).toEqual(expect.arrayContaining(['pt-nif', 'pt-niss', 'pt-citizen-card', 'credentials']));
    expect(sent[0]!.conditions.dictionaries).toContain('internal secret');
  });

  it('stored rules that are not a rule list are shown as text, not dropped', async () => {
    setup(policy({ dlpRulesJson: '{"legacy": true}' }));
    const text = await screen.findByRole('textbox', { name: /Stored rules \(text\)/ });
    expect(text).toHaveValue('{"legacy": true}');
    expect(screen.queryByTestId('dlp-editor')).toBeNull();
  });

  it('a server without the route shows Not available', async () => {
    const api: SecurityPolicyApi = { load: async () => ({ kind: 'unavailable' }), save: vi.fn() };
    renderInShell(() => <AdminSecurityPolicy api={api} />);
    expect(await screen.findByText(/Not available/)).toHaveTextContent('this server does not offer a security policy.');
  });

  it('has no axe violations and no inline style', async () => {
    const { container } = setup(policy({ envDlpRules: [{ ...RULE, id: 'env-1' } as never], pinnedByEnv: ['require2fa'] }));
    await screen.findByTestId('dlp-editor');
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });
});

describe('security policy values', () => {
  it('round-trips the wire shape', () => {
    const p = policy({ maxSecurityFloor: 'sanitized-no-media' });
    const w = policyWrite(policyValues(p));
    expect(w).toEqual({
      minTls: '1.2',
      require2fa: false,
      argon2MCost: 19456,
      argon2TCost: 2,
      argon2PCost: 1,
      capturePolicy: 'off',
      maxSecurityFloor: 'sanitized-no-media',
      dlpRulesJson: JSON.stringify([RULE]),
    });
  });

  it('validates rules', () => {
    expect(parseDlpRules('nope')).toBeNull();
    expect(parseDlpRules('[1]')).toBeNull();
    const r = newDlpRule([]);
    expect(validateDlpRules([{ ...r, name: 'x' }])).toBe('Rule 1 needs at least one thing to look for.');
    expect(validateDlpRules([{ ...r, name: 'x', conditions: { ...r.conditions, detectors: ['custom-regex'], customRegex: '(' } }])).toBe(
      'Rule 1: the pattern is not a valid regular expression.',
    );
    expect(validateDlpRules([{ ...r, name: 'x', conditions: { ...r.conditions, detectors: ['ssn'] } }])).toBeNull();
    expect(newDlpRule(['rule-2']).id).toBe('rule-3');
  });
});
