// V7 Admin → Plugins screen (SPEC §22, plan §2.6 / §3 e6). SCAFFOLD stub (e0):
// inert, importable, typecheck-green, NOT routed. e6 fills the registry
// approve/enable/capability-grant UI + the `allow_unsigned` persistent banner; e14
// mounts it under the admin panel (§19). This file does NOT touch the router.

import type { JSX } from 'solid-js';

export function AdminPlugins(): JSX.Element {
  return (
    <section data-screen="admin-plugins" aria-label="Plugins">
      <h2>Plugins</h2>
      <p>Plugin registry admin not yet implemented (t7 e6).</p>
    </section>
  );
}

export default AdminPlugins;
