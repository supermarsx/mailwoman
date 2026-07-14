// V7 Admin → Assist screen (SPEC §14/§19, plan §2.6 / §3 e6). SCAFFOLD stub (e0):
// inert, importable, typecheck-green, NOT routed. e6 fills the endpoint allowlist,
// capability locks, data-class ceilings, and the tenant-wide kill switch; e14
// mounts it under the admin panel. This file does NOT touch the router.

import type { JSX } from 'solid-js';

export function AdminAssist(): JSX.Element {
  return (
    <section data-screen="admin-assist" aria-label="Assist">
      <h2>Assist</h2>
      <p>Assist admin (endpoints, capability locks, data ceilings, kill switch) not yet implemented (t7 e6).</p>
    </section>
  );
}

export default AdminAssist;
