/** A readable account name with an unpredictable discovery suffix, not a password. */
export function randomAdminUsername(value = 'admin'): string {
  const prefix = value.replace(/-[0-9a-f]{12}$/i, '').trim().toLowerCase()
    .replace(/[^a-z0-9._@-]/g, '-').slice(0, 51).replace(/-+$/, '') || 'admin';
  const bytes = crypto.getRandomValues(new Uint8Array(6));
  return `${prefix}-${Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('')}`;
}
