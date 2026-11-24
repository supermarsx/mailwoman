import contract from './native-read-intent-contract.json';

// This is a declaration of a supported read shape, never an auth/session proof.
// Callers pass their existing request object; transport never reparses JSON.
const propertySet = new Set<string>(contract.properties);
const argumentSet = new Set<string>(contract.arguments);
const utf8 = new TextEncoder();
// eslint-disable-next-line no-control-regex -- matching control characters is the point of this check
const ascii = /^[\u0000-\u007f]*$/;
const byteLength = (value: string) => ascii.test(value) ? value.length : utf8.encode(value).byteLength;
const object = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
export function isNativeMetadataReadIntent(value: unknown): boolean {
  if (!object(value) || Object.keys(value).some(key => key !== 'using' && key !== 'methodCalls') || !Array.isArray(value.using) || value.using.length > 32 || value.using.some(capability => typeof capability !== 'string' || capability.length > 256) || !Array.isArray(value.methodCalls) || value.methodCalls.length !== 1) return false;
  const call = value.methodCalls[0];
  if (!Array.isArray(call) || call.length !== 3 || call[0] !== 'Email/get' || typeof call[2] !== 'string' || call[2].length === 0 || call[2].length > contract.maxCallIdBytes || byteLength(call[2]) > contract.maxCallIdBytes || !object(call[1])) return false;
  const args = call[1];
  if (Object.keys(args).some(key => !argumentSet.has(key)) || typeof args.accountId !== 'string' || args.accountId.length === 0 || args.accountId.length > contract.maxIdBytes || byteLength(args.accountId) > contract.maxIdBytes) return false;
  for (const key of ['mailwomanBodyUse', 'fetchHTMLBodyValues', 'fetchTextBodyValues', 'fetchAllBodyValues']) {
    if (key in args && args[key] !== false) return false;
  }
  if (!Array.isArray(args.ids) || args.ids.length === 0 || args.ids.length > contract.maxIds || args.ids.some(id => typeof id !== 'string' || id.length === 0 || id.length > contract.maxIdBytes || byteLength(id) > contract.maxIdBytes)) return false;
  if (!Array.isArray(args.properties) || args.properties.length !== propertySet.size) return false;
  const properties = new Set(args.properties);
  return properties.size === propertySet.size && [...properties].every(property => typeof property === 'string' && propertySet.has(property));
}
export function nativeReadIntentHeaders(value: unknown): Readonly<Record<string, string>> {
  return isNativeMetadataReadIntent(value) ? { [contract.header]: contract.value } : {};
}
