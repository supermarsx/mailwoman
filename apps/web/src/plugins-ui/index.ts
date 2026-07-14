// TypeScript UI-plugin tier — public surface (t10 plan §3 e10 / §2.3, SPEC §22.2).
//
// The tier is ADDITIVE + lazy: importing this module has no side effects, and the tier
// renders nothing when no plugin is approved (the mailbox path is byte-unchanged). e13
// lazy-mounts `UiPluginTier` in the app shell.
//
// Security core: plugins render ONLY inside an opaque-origin sandboxed iframe
// (`PLUGIN_IFRAME_SANDBOX` — `allow-scripts`, NO `allow-same-origin`); a deny-by-default
// postMessage broker (`classifyMessage`/`handleGuestMessage`) forwards only allow-listed
// capability methods (`CAP_METHOD_ALLOWLIST`, mirroring the e11 server `cap_methods`) to
// `POST /api/ui-plugins/{id}/rpc`.

export {
  RPC_PROTOCOL_VERSION,
  UI_CAPABILITIES,
  UI_EXTENSION_POINTS,
  EMPTY_REGISTRY,
  type RpcError,
  type RpcErrorCode,
  type RpcRequest,
  type RpcResponse,
  type UiCapability,
  type UiExtensionPoint,
  type UiPluginGrant,
  type UiPluginManifest,
  type UiPluginRegistration,
  type UiPluginRegistry,
} from './types';

export {
  CAP_METHOD_ALLOWLIST,
  LOCKED_PLUGIN_CSP,
  PLUGIN_IFRAME_SANDBOX,
  UiPluginHost,
  brokerReject,
  buildGuestSrcdoc,
  isTrustedGuestEvent,
  parseRpcRequest,
  rpcError,
  rpcErrorResponse,
} from './host';

export {
  attachBroker,
  classifyMessage,
  handleGuestMessage,
  type BrokerDecision,
  type BrokerWiring,
} from './broker';

export { callUiPluginRpc, listUiPlugins, makeRpcRequest } from './client';

export { UiPluginTier, UnsignedBanner, type UiPluginTierProps } from './Tier.tsx';
export { default } from './Tier.tsx';
