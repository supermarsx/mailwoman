// Keep literal imports visible to Rollup. No UI runtime import belongs here:
// both demanded panes and policy-gated idle prewarming share these loaders.
export const loadComposePane=()=>import('../components/Compose.tsx');
export const loadSettingsPane=()=>import('./Settings.tsx');
