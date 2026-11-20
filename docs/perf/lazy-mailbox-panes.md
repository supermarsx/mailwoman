# Deferred mailbox Compose and Settings panes

Mailbox imports small typed boundaries rather than the Compose and full Settings
runtime components. Shared reply/draft models, settings navigation/preferences and
ribbon styles remain where the mailbox needs them. Reader's existing inline reply
composer continues using its dynamic import.

Each demanded pane uses the existing LazyRoute import/error/timeout/retry/reload
contract. The boundary remains inside the original pane's command-ribbon scope,
navigation guard and detached DOM owner. Props are forwarded reactively; neither
the existing draft id nor the Settings navigation object is replaced on activation.
Closing a still-loading pane disposes its owner, so completing that import cannot
mount a late editor or publish orphan ribbon commands. Failed imports can retry
before any editor has mounted. Existing workspace tests continue to cover actual
Compose drafts, unsaved Settings navigation, detached panes and ribbon behavior;
new boundary tests use controlled deferred modules to exercise import timing.

The same literal import loaders are available to the existing policy-controlled
idle prefetch sequence. Disabling optional prefetch does not block user-requested
Compose/Settings navigation. Full offline prewarming still includes these chunks;
core mode retains its documented first-use limitations. CSP, sanitization,
service-worker digests and logical immutable asset URLs are unchanged.

Every successful main Vite build writes a private rendered-module graph to
`node_modules/.cache/mailwoman-build/bundle-graph.json`. The size checker verifies
its output directory and exact chunk SHA-256 values, then walks static and dynamic
edges. Compose.tsx and Settings.tsx must be rendered in reachable deferred chunks,
outside both the entry and Mailbox static closures. It deliberately permits shared
models/styles, and does not pretend all compose/settings files are absent.

For an alternate dist, supply its matching private graph as the second argument
to `scripts/check-size.mjs`. Old dist files and build transform-cache times do not
prove the new graph or browser startup gains. A current production build, graph/
size/CSP checks and real cold/warm browser navigation remain necessary before
claiming a startup speed improvement.
