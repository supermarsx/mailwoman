// V7 Assist (AI) module (SPEC §14, plan §2.6 / §3 e6). Public surface for e14 to
// wire into the mailbox + composer + settings. This module does NOT touch the
// router or Settings.tsx — ownership boundary (e7/e14 own those).
//
// HARD RULES (§14, R4):
//   - The whole Assist UI is HIDDEN when the gateway is Disabled (every component
//     is gated on `hasCapability` / `availability === 'enabled'`).
//   - NO Assist path transmits / deletes / accepts — send is ALWAYS human-gated.
//     There is no send capability and no send method on `AssistService`.
//
// e14 WIRE-UP (all lazy — this module is absent from the mailbox entry bundle):
//   import { createAssistSlice } from '@/state/slices/assist';   // reactive config + logs
//   const assist = createAssistSlice(ctx);  await assist.loadConfig();
//   <AssistPanel config={assist.config()} service={assist.service}
//                context={openThreadContext()} onReviewAction={openInComposer} />
//   <ComposerTools config={assist.config()} service={assist.service}
//                  text={body()} account={acct} onApply={setBody}
//                  onDisclosure={(d) => assist.recordDisclosure('draft', d)} />
//   <Dictation config={assist.config()} service={assist.service} onTranscript={appendToBody} />
//   <SemanticSearchToggle config={assist.config()} enabled={semantic()} onChange={setSemantic} />
//   <AutoTag config={assist.config()} messageId={id} suggestions={sugg}
//            mode={assist.autoTagMode()} onModeChange={assist.setAutoTagMode}
//            onApply={mail.addTag} onRevert={mail.removeTag} onAudit={assist.recordTagAudit} />
//
// Endpoints these call (e9 fills, e14 mounts):
//   GET  /api/assist/config       → WireAssistConfig  (availability + granted capabilities)
//   POST /api/assist/invoke       → WireInvokeResult  (server proxies + redacts; no send)
//   POST /api/assist/transcribe   → { text }          (Assist STT fallback for dictation)

export { AssistPanel, type AssistPanelProps } from './AssistPanel.tsx';
export { ComposerTools, type ComposerToolsProps } from './ComposerTools.tsx';
export { Dictation, type DictationProps } from './Dictation.tsx';
export { SemanticSearchToggle, type SemanticSearchToggleProps } from './SemanticSearchToggle.tsx';
export { AutoTag, type AutoTagProps } from './AutoTag.tsx';
export { Disclosure, type DisclosureProps } from './Disclosure.tsx';
export { AssistService, AssistError, type Fetcher } from './service.ts';
export {
  ASSIST_CAPABILITIES,
  COMPOSER_TOOLS,
  DISABLED_CONFIG,
  WHAT_LEFT_THE_DEVICE,
  configFromWire,
  disclosureSentence,
  hasCapability,
  type AssistAvailability,
  type AssistCapability,
  type AssistConfig,
  type AutoTagMode,
  type ChatMessage,
  type ComposerTool,
  type ComposerToolSpec,
  type ContentKind,
  type ContextItem,
  type Disclosure as DisclosureInfo,
  type InvokeRequest,
  type InvokeResult,
  type ProposedAction,
  type TagAuditEntry,
  type TagSuggestion,
  type WireAssistConfig,
} from './types.ts';
