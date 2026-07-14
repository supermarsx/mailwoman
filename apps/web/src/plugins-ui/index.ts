// TypeScript UI-plugin tier — public surface (t10 plan §2.3, SPEC §22.2). Inert +
// NOT routed: e10 lazy-mounts the tier once a plugin is approved. Importing this
// module has no side effects.

export {
  RPC_PROTOCOL_VERSION,
  UI_CAPABILITIES,
  UI_EXTENSION_POINTS,
  type RpcError,
  type RpcErrorCode,
  type RpcRequest,
  type RpcResponse,
  type UiCapability,
  type UiExtensionPoint,
  type UiPluginGrant,
  type UiPluginManifest,
  type UiPluginRegistration,
} from './types';

export {
  CAP_METHOD_ALLOWLIST,
  PLUGIN_IFRAME_SANDBOX,
  UiPluginHost,
  brokerReject,
  brokerRespond,
  rpcError,
} from './host';
