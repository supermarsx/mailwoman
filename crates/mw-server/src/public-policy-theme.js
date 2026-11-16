(() => {
  'use strict';
  const root = document.documentElement;
  const allowed = {bg:'color',surface:'color',text:'color',dim:'color',border:'color',accent:'color',link:'color',radius:'border-radius',font:'font-family'};
  let active = null;
  function apply(value) {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return;
    const palette = {};
    for (const [name, property] of Object.entries(allowed)) {
      const color = value[name];
      if (typeof color === 'string' && color.length <= 256 && !/[\x00-\x1f\x7f]/.test(color) && CSS.supports(property, color)) {
        root.style.setProperty('--policy-' + name, color);
        palette[name] = color;
      }
    }
    if (value.scheme === 'dark' || value.scheme === 'light') { root.style.colorScheme=value.scheme; palette.scheme=value.scheme; }
    active = palette;
    try { history.replaceState({...history.state, policyTheme:palette}, ''); } catch { /* Appearance remains usable. */ }
  }
  try {
    const params = new URLSearchParams(location.hash.slice(1));
    const raw = params.get('mw-theme');
    if (raw && raw.length <= 8192) {
      apply(JSON.parse(raw));
      params.delete('mw-theme');
      history.replaceState(history.state, '', location.pathname + location.search + (params.size ? '#' + params : ''));
    } else apply(history.state?.policyTheme);
  } catch { /* Invalid appearance data cannot break a public document. */ }
  addEventListener('hashchange', () => { if (active) apply(active); });
  try {
    const channel = new BroadcastChannel('mw-public-policy-theme');
    channel.onmessage = event => { if (event.data?.type === 'policy-theme') apply(event.data.palette); };
    channel.postMessage({type:'request-theme'});
    addEventListener('pagehide', event => { if (!event.persisted) channel.close(); });
  } catch { /* Direct visits and older browsers retain their system theme. */ }
})();
