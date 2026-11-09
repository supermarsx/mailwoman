import { providerDeliveryStatus, type DeliveryStatusContext, type ProviderMeaning } from './providerDeliveryStatus.ts';
/** Registered enhanced SMTP detail codes, including deprecated x.6.10.
 * Source: https://www.iana.org/assignments/smtp-enhanced-status-codes/
 * Checked 2026-11-09. Provider-specific codes use diagnostic context before the standard fallback.
 */
const registered = new Set(['0.0', '1.0', '1.1', '1.2', '1.3', '1.4', '1.5', '1.6', '1.7', '1.8', '1.9', '1.10', '2.0', '2.1', '2.2', '2.3', '2.4', '3.0', '3.1', '3.2', '3.3', '3.4', '3.5', '3.6', '4.0', '4.1', '4.2', '4.3', '4.4', '4.5', '4.6', '4.7', '5.0', '5.1', '5.2', '5.3', '5.4', '5.5', '5.6', '6.0', '6.1', '6.2', '6.3', '6.4', '6.5', '6.6', '6.7', '6.8', '6.9', '6.10', '7.0', '7.1', '7.2', '7.3', '7.4', '7.5', '7.6', '7.7', '7.8', '7.9', '7.10', '7.11', '7.12', '7.13', '7.14', '7.15', '7.16', '7.17', '7.18', '7.19', '7.20', '7.21', '7.22', '7.23', '7.24', '7.25', '7.26', '7.27', '7.28', '7.29', '7.30']);
export function deliveryStatusMeaning(status: string, context: DeliveryStatusContext = {}): {classKey: string; reasonKey: string; adviceKey: string; specific: boolean; provider?: ProviderMeaning} | null {
  const match = /^([245])\.(\d{1,3})\.(\d{1,3})$/.exec(status);
  if (!match) return null;
  const [,kind,subject,detail] = match;
  const suffix = `${subject}.${detail}`;
  const provider = providerDeliveryStatus(status,context);
  const specific = !!provider || registered.has(suffix);
  const category = /^[0-7]$/.test(subject!) ? subject : '0';
  return {
    classKey: `dsn-class-${kind}`,
    reasonKey: provider?.reasonKey ?? (specific ? `dsn-status-${subject}-${detail}` : `dsn-category-${category}`),
    adviceKey: provider?.adviceKey ?? (kind === '2' ? 'dsn-advice-success' : `dsn-advice-${category}`),
    ...(provider ? {provider} : {}),
    specific,
  };
}
