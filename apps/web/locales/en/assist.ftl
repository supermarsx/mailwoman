# Mailwoman — Assist (AI) strings (source locale: en). Ids prefixed `assist-`.
#
# HONESTY: the "what left the device" disclosure (assist-left-*, assist-disclosure-*)
# is a concrete, factual statement of exactly what Assist can and cannot send, and
# that send is never automated. It is byte-compatible with the canonical wording in
# `src/modules/assist/types.ts` / `Disclosure.tsx`. Do NOT weaken it when translating:
# keep "never sends …", "excluded by default", and "Send is never automated" accurate.

# -- Assist chat panel ------------------------------------------------------
assist-panel-label = Assist
assist-heading = Assistant
assist-transcript-label = Assistant conversation
assist-proposed-action = Proposed action
assist-outbox-note = This would place a message in your Outbox. Nothing is sent until you confirm it there.
assist-review-outbox = Review in Outbox
assist-review = Review
assist-empty = Ask the assistant to summarise, draft, or find things. It can propose actions, but you always confirm them yourself.
assist-error = The assistant could not respond just now.
assist-input-label = Message the assistant
assist-input-placeholder = Ask the assistant…
assist-thinking = Thinking…
assist-ask = Ask

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Writing help
assist-composer-toolbar = Composer tools
assist-tool-grammar = Fix grammar
assist-tool-rewrite = Rewrite
assist-tool-tone = Adjust tone
assist-tool-translate = Translate
assist-toolarg-tone = Tone
assist-toolarg-translate = Language
assist-busy = { $label }…
assist-composer-empty-error = Write something first.
assist-composer-error = Assist could not complete that just now.
assist-suggested-edit = Suggested edit
assist-apply-draft = Apply to draft
assist-discard = Discard

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Suggested labels
assist-apply-auto = Apply automatically
assist-apply-label = Apply { $label }
assist-remove-label = Remove { $label }
assist-apply = Apply
assist-undo = Undo

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Hold to dictate
assist-dictate-stop = Stop dictation
assist-dictate-hold-text = 🎙 Hold to dictate
assist-dictate-listening = ● Listening…
assist-dictate-endpoint-note = Audio is transcribed by your Assist endpoint.
assist-dictate-err = Dictation error.
assist-transcribe-err = Could not transcribe audio.
assist-mic-err = Microphone unavailable.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Semantic search
assist-semantic-note = Query text is sent to your Assist endpoint to rank by meaning.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = What can leave this device
assist-left-1 = the selected message text (subject + body) for the chosen capability
assist-left-2 = the endpoint host it was sent to
assist-left-3 = never: end-to-end-encrypted content (excluded by default)
assist-left-4 = never: attachments (excluded by default)
assist-left-5 = never: your credentials or other accounts
assist-disclosure-off = Assist is off. No message content leaves this device.
assist-disclosure-sentence = When you use an Assist tool, the selected message text is proxied to { $host }. { $excl } Send is never automated — you always confirm before anything leaves your Outbox.
assist-disclosure-excl-one = It never sends { $a }.
assist-disclosure-excl-two = It never sends { $a } or { $b }.
assist-disclosure-admin-allowed = Your admin has allowed encrypted content and attachments to be sent.
assist-excluded-e2ee = end-to-end-encrypted content
assist-excluded-attachments = attachments

# -- Admin → Assist (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = Assist
assist-admin-intro = Assist sends message text that users select to the AI endpoint configured here. It cannot send, delete, or accept mail. End-to-end-encrypted content and attachments are withheld unless you allow them below.
assist-admin-load-error = Could not load the Assist configuration. Saving is disabled so the stored configuration is not overwritten.
assist-admin-status-running = Assist is running. Requests go to { $host }.
assist-admin-status-running-pending = Assist is running with the configuration the server started with. Saved changes take effect when the server restarts.
assist-admin-status-off = Assist is off. Every Assist request is refused.
assist-admin-status-pending = Assist is not running. The saved configuration takes effect when the server restarts.
assist-admin-status-no-endpoint = Assist is on but not running because no usable endpoint is configured.
assist-admin-stop = Stop Assist now
assist-admin-stop-note = Stopping takes effect immediately for every user and is kept across restarts. Assist controls stay visible to users until the server restarts; requests made with them are refused.
assist-admin-resume = Turn Assist on
assist-admin-resume-note = If the server started with Assist off, turning it on takes effect when the server restarts.
assist-admin-kill-error = Could not change whether Assist is on.
assist-admin-endpoint = Endpoint
assist-admin-endpoint-note = The server contacts this endpoint; browsers never do. Changes take effect when the server restarts.
assist-admin-kind = Endpoint type
assist-admin-kind-none = None
assist-admin-kind-open-ai-compatible = OpenAI-compatible
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Local process
assist-admin-local-process-note = This deployment runs the program { $program } for Assist. It can be changed through the admin API, not on this screen.
assist-admin-base-url = Base URL
assist-admin-base-url-required = Enter the endpoint's base URL, starting with http:// or https://.
assist-admin-api-key = API key
assist-admin-chat-model = Chat model
assist-admin-embed-model = Embedding model
assist-admin-stt-model = Speech-to-text model
assist-admin-model = Model
assist-admin-anthropic-version = API version
assist-admin-max-tokens = Maximum reply tokens
assist-admin-default-placeholder = Server default
assist-admin-grants = Capabilities
assist-admin-grants-note = Only the capabilities checked here can be used.
assist-admin-cap-summarize = Summarize
assist-admin-cap-draft = Draft and rewrite
assist-admin-cap-grammar = Grammar
assist-admin-cap-dictation = Dictation
assist-admin-cap-search-semantic = Semantic search
assist-admin-cap-auto-tag = Auto-tag
assist-admin-cap-recap = Recap
assist-admin-cap-assistant = Assistant chat
assist-admin-ceilings = Data limits
assist-admin-ceilings-note = These limits apply to every request, whatever the capability.
assist-admin-accounts = Account ids whose mail may be sent, one per line
assist-admin-accounts-note = With no account listed, no mail content is sent to the endpoint.
assist-admin-folders = Folder ids to limit sending to, one per line
assist-admin-folders-note = With no folder listed, every folder of the listed accounts is allowed.
assist-admin-allow-e2ee = Allow end-to-end-encrypted content to be sent
assist-admin-allow-attachments = Allow attachments to be sent
assist-admin-save = Save configuration
assist-admin-saved = Saved.
assist-admin-saved-pending = Saved. The change takes effect when the server restarts.
assist-admin-save-error = The configuration was not saved.
