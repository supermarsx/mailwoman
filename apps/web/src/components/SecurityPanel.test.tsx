import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { SecurityPanel } from './SecurityPanel.tsx';
import type {
  AttachmentRiskKind,
  AuthResult,
  SecurityAnomaly,
  SecurityVerdict,
  SignatureStatus,
} from '../api/security-types.ts';

// A fully-passing baseline verdict; tests override single facets so each state is
// exercised against the FROZEN shape (plan §3 e3 acceptance).
function baseVerdict(): SecurityVerdict {
  return {
    emailId: 'M1',
    plainLanguage: 'This message passed all authentication checks.',
    auth: {
      dkim: { result: 'pass', domain: 'example.com', selector: 'sel1' },
      spf: { result: 'pass', domain: 'example.com' },
      dmarc: { result: 'pass', policy: 'reject', aligned: true },
      arc: { result: 'pass', chainLength: 1 },
    },
    received: [
      {
        index: 0,
        byHost: 'mx.example.com',
        fromHost: 'sender.example.net',
        protocol: 'ESMTPS',
        timestamp: '2026-07-11T10:00:00Z',
        delayMs: 1200,
        asn: 15169,
        asnOrg: 'Example ISP',
        country: 'US',
      },
    ],
    signature: {
      kind: 'pgp',
      status: 'verified',
      signerKeyId: 'ABCD1234',
      algorithm: 'ed25519',
      keyCreatedAt: '2024-01-01T00:00:00Z',
      keyExpiresAt: '2027-01-01T00:00:00Z',
      chainStatus: 'trusted',
      revocationStatus: 'good',
      keyChanged: false,
    },
    encryption: { kind: 'none', isEncrypted: false, decryptsClientSide: false },
    attachments: [],
    anomalies: [],
  };
}

function renderExpanded(overrides: Partial<SecurityVerdict> = {}, props = {}) {
  return render(() => (
    <SecurityPanel verdict={{ ...baseVerdict(), ...overrides }} initiallyExpanded {...props} />
  ));
}

describe('SecurityPanel — collapsed chip', () => {
  it('renders the plain-language chip collapsed by default (panel hidden)', () => {
    render(() => <SecurityPanel verdict={baseVerdict()} />);
    const chip = screen.getByRole('button', { expanded: false });
    expect(chip).toHaveTextContent('This message passed all authentication checks.');
    expect(screen.queryByRole('region')).toBeNull();
  });

  it('expands and collapses on click, toggling aria-expanded', () => {
    render(() => <SecurityPanel verdict={baseVerdict()} />);
    const chip = screen.getByRole('button');
    fireEvent.click(chip);
    expect(chip).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByRole('region', { name: 'Message security details' })).toBeInTheDocument();
    fireEvent.click(chip);
    expect(chip).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('region')).toBeNull();
  });

  it('colours the chip by the worst facet (good / warning / bad)', () => {
    const { container: good } = render(() => <SecurityPanel verdict={baseVerdict()} />);
    expect(good.querySelector('[data-tone]')).toHaveAttribute('data-tone', 'good');

    const bad = baseVerdict();
    bad.auth.dkim.result = 'fail';
    const { container: badC } = render(() => <SecurityPanel verdict={bad} />);
    expect(badC.querySelector('[data-tone]')).toHaveAttribute('data-tone', 'bad');

    const warn = baseVerdict();
    warn.anomalies = ['replyToMismatch'];
    const { container: warnC } = render(() => <SecurityPanel verdict={warn} />);
    expect(warnC.querySelector('[data-tone]')).toHaveAttribute('data-tone', 'warning');
  });
});

describe('SecurityPanel — authentication', () => {
  const RESULTS: AuthResult[] = ['pass', 'fail', 'none', 'neutral', 'temperror', 'permerror'];
  const LABEL: Record<AuthResult, string> = {
    pass: 'passed',
    fail: 'failed',
    none: 'not present',
    neutral: 'neutral',
    temperror: 'temporary error',
    permerror: 'permanent error',
  };

  for (const result of RESULTS) {
    it(`renders every auth mechanism for result "${result}"`, () => {
      const v = baseVerdict();
      v.auth.dkim.result = result;
      v.auth.spf.result = result;
      v.auth.dmarc.result = result;
      v.auth.arc.result = result;
      renderExpanded(v);
      for (const mech of ['DKIM', 'SPF', 'DMARC', 'ARC']) {
        expect(screen.getByText(`${mech} ${LABEL[result]}`)).toBeInTheDocument();
      }
    });
  }

  it('shows expert alignment detail (domain / selector / policy / alignment)', () => {
    renderExpanded();
    expect(screen.getByText(/domain: example\.com · selector: sel1/)).toBeInTheDocument();
    expect(screen.getByText(/policy: reject · alignment: aligned/)).toBeInTheDocument();
  });
});

describe('SecurityPanel — received chain', () => {
  it('renders hops with delay and optional ASN/country', () => {
    renderExpanded();
    expect(screen.getByText(/sender\.example\.net → mx\.example\.com/)).toBeInTheDocument();
    expect(screen.getByText('+1.2 s')).toBeInTheDocument();
    expect(screen.getByText(/AS15169 Example ISP · US/)).toBeInTheDocument();
  });

  it('falls back when there is no Received chain', () => {
    renderExpanded({ received: [] });
    expect(screen.getByText('No Received chain available.')).toBeInTheDocument();
  });
});

describe('SecurityPanel — signature (3-state)', () => {
  const CASES: Record<SignatureStatus, string> = {
    verified: 'Signature verified',
    'unverified-key': 'Signed — signer key not verified',
    invalid: 'Signature is invalid',
    none: 'Not signed',
  };

  for (const status of ['verified', 'unverified-key', 'invalid'] as SignatureStatus[]) {
    it(`renders signature status "${status}"`, () => {
      const v = baseVerdict();
      v.signature = { ...v.signature!, status };
      renderExpanded(v);
      expect(screen.getByText(new RegExp(CASES[status]))).toBeInTheDocument();
    });
  }

  it('renders the "not signed" state when signature is null', () => {
    renderExpanded({ signature: null });
    // Section fallback text.
    const region = screen.getByRole('region');
    expect(region).toHaveTextContent('Not signed');
  });

  it('surfaces chain / revocation / key-change detail', () => {
    const v = baseVerdict();
    v.signature = {
      ...v.signature!,
      chainStatus: 'expired',
      revocationStatus: 'revoked',
      keyChanged: true,
    };
    renderExpanded(v);
    expect(screen.getByText('Chain expired')).toBeInTheDocument();
    expect(screen.getByText('Key revoked')).toBeInTheDocument();
    expect(screen.getByText('Signer key changed since last seen')).toBeInTheDocument();
  });
});

describe('SecurityPanel — attachment risk', () => {
  const CASES: Record<AttachmentRiskKind, string> = {
    none: 'No known risk',
    macro: 'Contains macros',
    executable: 'Executable file',
    'encrypted-archive': 'Encrypted archive',
    'double-extension': 'Double file extension',
  };

  for (const risk of Object.keys(CASES) as AttachmentRiskKind[]) {
    it(`flags attachment risk "${risk}"`, () => {
      renderExpanded({
        attachments: [
          { name: `file-${risk}`, declaredType: 'application/pdf', detectedType: null, mismatch: false, risk },
        ],
      });
      expect(screen.getByText(CASES[risk])).toBeInTheDocument();
      expect(screen.getByText(`file-${risk}`)).toBeInTheDocument();
    });
  }

  it('notes an extension-vs-magic mismatch', () => {
    renderExpanded({
      attachments: [
        {
          name: 'invoice.pdf',
          declaredType: 'application/pdf',
          detectedType: 'application/x-dosexec',
          mismatch: true,
          risk: 'executable',
        },
      ],
    });
    expect(
      screen.getByText(/type mismatch \(declared application\/pdf, detected application\/x-dosexec\)/),
    ).toBeInTheDocument();
  });

  it('shows an empty state with no attachments', () => {
    renderExpanded();
    expect(screen.getByText('No attachments.')).toBeInTheDocument();
  });
});

describe('SecurityPanel — anomalies', () => {
  const TOKENS: Record<SecurityAnomaly, string> = {
    replyToMismatch: 'Reply-To address differs from the sender',
    envelopeFromDivergence: 'Envelope sender differs from the From address',
    messageIdDomainAnomaly: "Message-ID domain doesn't match the sender",
    dateSkew: 'Send date looks skewed',
    punycodeSender: 'Sender uses punycode (possible look-alike domain)',
  };

  for (const token of Object.keys(TOKENS) as SecurityAnomaly[]) {
    it(`renders anomaly token "${token}"`, () => {
      renderExpanded({ anomalies: [token] });
      expect(screen.getByText(TOKENS[token])).toBeInTheDocument();
    });
  }

  it('renders no Warnings section when there are no anomalies', () => {
    renderExpanded();
    expect(screen.queryByText('Warnings')).toBeNull();
  });
});

describe('SecurityPanel — sender controls', () => {
  it('renders every control button', () => {
    renderExpanded();
    for (const name of [
      'Block sender',
      'Silence sender',
      'Ignore conversation',
      'Report phishing',
      'Report junk',
    ]) {
      expect(screen.getByRole('button', { name })).toBeInTheDocument();
    }
  });

  it('block dispatches SenderControl/set with the sender address', () => {
    const onSenderControl = vi.fn().mockResolvedValue({ updated: true, mailRuleId: 'r1' });
    renderExpanded({}, { senderAddress: 'evil@spam.example', onSenderControl });
    fireEvent.click(screen.getByRole('button', { name: 'Block sender' }));
    expect(onSenderControl).toHaveBeenCalledWith({
      action: 'block',
      emailId: 'M1',
      address: 'evil@spam.example',
    });
  });

  it('ignore-conversation dispatches with the threadId', () => {
    const onSenderControl = vi.fn().mockResolvedValue({ updated: true });
    renderExpanded({}, { senderAddress: 'a@b.example', threadId: 'T9', onSenderControl });
    fireEvent.click(screen.getByRole('button', { name: 'Ignore conversation' }));
    expect(onSenderControl).toHaveBeenCalledWith({
      action: 'ignore-conversation',
      emailId: 'M1',
      threadId: 'T9',
    });
  });

  it('report actions set abuseReport', () => {
    const onSenderControl = vi.fn().mockResolvedValue({ updated: true });
    renderExpanded({}, { senderAddress: 'a@b.example', onSenderControl });
    fireEvent.click(screen.getByRole('button', { name: 'Report phishing' }));
    expect(onSenderControl).toHaveBeenCalledWith({
      action: 'report-phishing',
      emailId: 'M1',
      address: 'a@b.example',
      abuseReport: true,
    });
  });

  it('confirms the action in the status live region', async () => {
    renderExpanded({}, { senderAddress: 'a@b.example' });
    fireEvent.click(screen.getByRole('button', { name: 'Silence sender' }));
    expect(await screen.findByText('Sender silenced')).toBeInTheDocument();
  });

  it('falls back to the built-in mock when no handler is supplied', async () => {
    renderExpanded({}, { senderAddress: 'a@b.example' });
    fireEvent.click(screen.getByRole('button', { name: 'Block sender' }));
    // The default mock resolves and the confirmation appears without a handler prop.
    expect(await screen.findByText('Sender blocked')).toBeInTheDocument();
  });
});
