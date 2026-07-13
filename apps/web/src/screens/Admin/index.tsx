// V6 Admin panel screen (SPEC §19, plan §2.6, §3 e7). SCAFFOLD (t6-e0): an inert,
// lazily-loadable placeholder — NOT wired into any route yet, so the normal
// mailbox bundle is byte-unchanged. e7 fills the §19 panel (Domains / Users /
// Security-policy / Integrations / Observability / Appearance) talking to the
// e5/e11 admin endpoints, gated on an admin session, and registers it as a lazy
// route (`lazy(() => import('./screens/Admin'))`). The default export makes it
// directly `lazy()`-loadable.

import type { JSX } from 'solid-js';

/** Placeholder admin screen. Replaced by e7 with the real §19 panel. */
export function AdminScreen(): JSX.Element {
  return (
    <section aria-label="Admin panel" data-screen="admin">
      <h1>Admin panel</h1>
      <p>The Mailwoman admin panel arrives in V6.</p>
    </section>
  );
}

export default AdminScreen;
