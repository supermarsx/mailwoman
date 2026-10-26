import type { DomainBackend } from '../../../api/admin/domains.ts';

export function ewsBackendProblem(row: DomainBackend): string | null {
  const config = row.ews;
  if (!config || config.authentication !== 'basic' || row.endpoint || row.sessionUrl || row.role !== 'mail') return 'backend-ews-invalid';
  try {
    const value = config.endpointUrl;
    const url = new URL(value);
    if (value.length > 2048 || [...value].some(char => char.charCodeAt(0) <= 32 || char.charCodeAt(0) === 127) || url.protocol !== 'https:' || !url.hostname || url.username || url.password || value.includes('?') || value.includes('#')) return 'backend-ews-invalid';
  } catch { return 'backend-ews-invalid'; }
  if (config.caPem !== undefined) {
    const pem = config.caPem;
    const certificates = pem.match(/-----BEGIN CERTIFICATE-----[\s\S]*?-----END CERTIFICATE-----/gu) ?? [];
    if (!pem.trim() || new TextEncoder().encode(pem).length > 65536 || pem.includes('PRIVATE KEY') || certificates.length < 1 || certificates.length > 8 || pem.replace(/-----BEGIN CERTIFICATE-----[\s\S]*?-----END CERTIFICATE-----/gu, '').trim()) return 'backend-ews-invalid-ca';
  }
  return null;
}

/** Same preferred, priority, ID selection order as the server. */
export function ewsSelectionProblem(rows: readonly DomainBackend[]): string | null {
  const mail = rows.filter(row => row.role === 'mail' && row.enabled !== false).sort((a, b) => Number(!!b.preferred) - Number(!!a.preferred) || (a.priority ?? 100) - (b.priority ?? 100) || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))[0];
  return mail?.protocol === 'ews' && rows.some(row => row.enabled !== false && (row.role === 'submission' || row.role === 'rules')) ? 'backend-ews-service-conflict' : null;
}
