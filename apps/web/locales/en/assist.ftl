# Mailwoman — AI assistant strings (source locale: en). Ids prefixed `assist-`.
# The user-visible name is "AI assistant"; the ids, routes and module keep `assist`.
#
# HONESTY: the "what left the device" disclosure (assist-left-*, assist-disclosure-*)
# is a concrete, factual statement of exactly what the AI assistant can and cannot send, and
# that send is never automated. It is byte-compatible with the canonical wording in
# `src/modules/assist/types.ts` / `Disclosure.tsx`. Do NOT weaken it when translating:
# keep "never sends …", "excluded by default", and "Send is never automated" accurate.

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = AI assistant
assist-heading = AI assistant
assist-transcript-label = AI assistant conversation
assist-proposed-action = Proposed action
assist-outbox-note = This would place a message in your Outbox. Nothing is sent until you confirm it there.
assist-review-outbox = Review in Outbox
assist-review = Review
assist-empty = Ask the AI assistant to summarise, draft, or find things. It can propose actions, but you always confirm them yourself.
assist-error = The AI assistant could not respond just now.
assist-input-label = Message the AI assistant
assist-input-placeholder = Ask the AI assistant…
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
assist-tone-neutral = Neutral
assist-tone-friendly = Friendly
assist-tone-formal = Formal
assist-tone-concise = Concise
assist-tone-assertive = Assertive
assist-busy = { $label }…
assist-composer-empty-error = Write something first.
assist-composer-error = The AI assistant could not complete that just now.
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
assist-dictate-hold-text = Hold to dictate
assist-dictate-listening = ● Listening…
assist-dictate-endpoint-note = Audio is transcribed by your AI assistant endpoint.
assist-dictate-err = Dictation error.
assist-transcribe-err = Could not transcribe audio.
assist-mic-err = Microphone unavailable.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Semantic search
assist-semantic-note = Query text is sent to your AI assistant endpoint to rank by meaning.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = What can leave this device
assist-left-1 = the selected message text (subject + body) for the chosen capability
assist-left-2 = the endpoint host it was sent to
assist-left-3 = never: end-to-end-encrypted content (excluded by default)
assist-left-4 = never: attachments (excluded by default)
assist-left-5 = never: your credentials or other accounts
assist-disclosure-off = The AI assistant is off. No message content leaves this device.
assist-disclosure-sentence = When you use an AI assistant tool, the selected message text is proxied to { $host }. { $excl } Send is never automated — you always confirm before anything leaves your Outbox.
assist-disclosure-excl-one = It never sends { $a }.
assist-disclosure-excl-two = It never sends { $a } or { $b }.
assist-disclosure-admin-allowed = Your admin has allowed encrypted content and attachments to be sent.
assist-excluded-e2ee = end-to-end-encrypted content
assist-excluded-attachments = attachments

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = AI assistant
assist-admin-intro = The AI assistant sends message text that users select to the AI endpoint configured here. It cannot send, delete, or accept mail. End-to-end-encrypted content and attachments are withheld unless you allow them below.
assist-admin-load-unreadable = The stored AI assistant configuration cannot be read: its API key does not open with this server's key (MW_SERVER_KEY). It was saved under another key or has been changed. The AI assistant is off. Fill in the form, including the API key, and save to replace it.
assist-admin-load-error = Could not load the AI assistant configuration. Saving is disabled so the stored configuration is not overwritten.
assist-admin-status-running = The AI assistant is running. Requests go to { $host }.
assist-admin-status-running-pending = The AI assistant is running with the configuration the server started with. Saved changes take effect when the server restarts.
assist-admin-status-off = The AI assistant is off. Every AI assistant request is refused.
assist-admin-status-pending = The AI assistant is not running. The saved configuration takes effect when the server restarts.
assist-admin-status-local-program-not-allowed = The AI assistant is not running. The configured program { $program } is not listed in MW_ASSIST_LOCAL_PROGRAMS on the server, so the server did not start it.
assist-admin-status-no-endpoint = The AI assistant is on but not running because no usable endpoint is configured.
assist-admin-stop = Stop the AI assistant now
assist-admin-stop-note = Stopping takes effect immediately for every user and is kept across restarts.
assist-admin-resume = Turn the AI assistant on
assist-admin-resume-note = If the server started with the AI assistant off, turning it on takes effect when the server restarts.
assist-admin-kill-error = Could not change whether the AI assistant is on.
assist-admin-endpoint = Endpoint
assist-admin-endpoint-note = The server contacts this endpoint; browsers never do. Changes take effect when the server restarts.
assist-admin-kind = Endpoint type
assist-admin-kind-none = None
assist-admin-kind-open-ai-compatible = OpenAI-compatible
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Local program
assist-admin-local-program-enable = To have the AI assistant run a program on this server, set MW_ASSIST_LOCAL_PROGRAMS to the program's absolute path where the server is started (separate several paths with commas), then restart the server.
assist-admin-local-program-note = The server starts this program for each AI assistant request and passes the request to it on standard input, including the message text the user selected. It reads the reply from the program's standard output. Only programs listed in MW_ASSIST_LOCAL_PROGRAMS on the server can be chosen.
assist-admin-local-program = Program
assist-admin-local-program-unlisted = { $program } (not listed)
assist-admin-local-program-unlisted-note = This program is not listed in MW_ASSIST_LOCAL_PROGRAMS, so the server does not run it and will not accept it when you save. Choose a listed program or another endpoint type.
assist-admin-local-program-required = Choose the program to run.
assist-admin-local-args = Arguments, one per line
assist-admin-local-args-note = Each line is passed to the program as one argument. No shell is involved.
assist-admin-base-url = Base URL
assist-admin-base-url-required = Enter the endpoint's base URL, starting with http:// or https://.
assist-admin-api-key = API key
assist-admin-api-key-stored = A key is stored. The server does not show it again.
assist-admin-api-key-replace = Replace the API key
assist-admin-api-key-replace-placeholder = Leave empty to keep the stored key
assist-admin-api-key-remove = Remove the stored key
assist-admin-api-key-keep = Keep the stored key
assist-admin-api-key-remove-pending = The stored key will be removed when you save.
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
assist-admin-cap-assistant = AI assistant chat
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
assist-admin-save-not-allowed = The configuration was not saved because the server does not allow this program. Only programs listed in MW_ASSIST_LOCAL_PROGRAMS can be used.

# Rebuilt on the settings form (f33, t32-cfg §4.12).
assist-admin-unavailable = this server does not offer the AI assistant settings.
assist-admin-off = Off
assist-admin-kind-help = Where the server sends AI assistant requests; None turns the endpoint off without stopping the assistant.
assist-admin-kind-help-no-local = Where the server sends AI assistant requests. A local program can be chosen once MW_ASSIST_LOCAL_PROGRAMS lists one where the server starts.
assist-admin-base-url-help = The address of the endpoint's API; required for an OpenAI-compatible endpoint, optional for Anthropic.
assist-admin-api-key-help = Write-only: the server stores it sealed and never shows it again; changing the type or base URL drops a stored key.
assist-admin-chat-model-help = The model used for summaries, drafts and chat; empty uses the server default.
assist-admin-embed-model-help = The model used for semantic search; empty uses the server default.
assist-admin-stt-model-help = The model used for dictation; empty uses the server default.
assist-admin-model-help = The Anthropic model; empty uses the server default.
assist-admin-anthropic-version-help = The anthropic-version header sent with each request; empty uses the server default.
assist-admin-max-tokens-help = The longest reply asked for; empty uses the server default.
assist-admin-local-args-label = Arguments
assist-admin-accounts-label = Account ids whose mail may be sent
assist-admin-accounts-none = No account: no mail content is sent.
assist-admin-folders-label = Folder ids to limit sending to
assist-admin-folders-none = No folder: every folder of the listed accounts.
assist-admin-allow-e2ee-help = Off withholds end-to-end-encrypted content from every request.
assist-admin-allow-attachments-help = Off withholds attachments from every request.
assist-admin-rate-heading = Rate limit
assist-admin-rate-group-help = Requests per minute the server accepts for the AI assistant across all users.
assist-admin-rate-unavailable = This server does not offer the rate limit in the console yet; it is set with MW_ASSIST_RATE_LIMIT_PER_MIN where the server starts.
assist-admin-rate-limit = Requests per minute
assist-admin-rate-limit-help = 0 means no limit; at most 10000.
assist-admin-rate-none = 0 (no limit)
assist-admin-stop-title = Stop the AI assistant for everyone?
assist-admin-stop-body = Every AI assistant request is refused from now on, for every user, until it is turned on again. The configuration is kept.
assist-admin-stopped-done = The AI assistant is stopped.
assist-admin-resumed-done = The AI assistant is turned on.

## Admin → AI assistant → provider profiles (t39 N8)
assist-profiles-title = Provider profiles
assist-profiles-intro = A profile is one AI provider endpoint, model, API key and request options. Assign a profile to the whole deployment, to a domain or to one user. With no profile assigned, the endpoint above is used. Every profile request goes through the outbound address checks: a profile cannot reach a private or local address unless that address is granted below.
assist-profiles-loading = Loading profiles…
assist-profiles-empty = No profiles yet.
assist-profiles-col-name = Name
assist-profiles-col-provider = Provider
assist-profiles-col-endpoint = Endpoint
assist-profiles-col-model = Model
assist-profiles-col-key = API key
assist-profiles-col-egress = Address access
assist-profiles-col-actions = Actions
assist-profiles-key-stored = Stored
assist-profiles-key-none = None
assist-profiles-egress-granted = Local address granted
assist-profiles-egress-public = Public addresses only
assist-profiles-test = Test profile
assist-profiles-test-running = Testing…
assist-profiles-test-ok = Answered in { $ms } ms.
assist-profiles-test-failed = Failed after { $ms } ms: { $reason } { $detail }
assist-profiles-edit = Edit
assist-profiles-delete = Delete
assist-profiles-delete-confirm = Delete { $name }? Users assigned to it lose the AI assistant until they are assigned another profile.
assist-profiles-cancel = Cancel
assist-profiles-add = Add profile
assist-profiles-new-title = New profile
assist-profiles-edit-title = Edit { $name }
assist-profiles-preset = Preset
assist-profiles-verified-mock = Tested against a local mock of this request format, not against the provider’s live service.
assist-profiles-unverified = Not tested. Check the endpoint and model name with the provider, then use Test profile.
assist-profiles-name = Name
assist-profiles-endpoint = Endpoint URL
assist-profiles-needs-grant = This endpoint is on this machine or the local network. Requests to { $destination } are refused until it is granted under Local endpoints.
assist-profiles-model = Model
assist-profiles-api-key = API key
assist-profiles-key-stored-help = A key is stored. Leave the field empty to keep it, or type a new key to replace it. The stored key is never shown.
assist-profiles-remove-key = Remove the stored key
assist-profiles-options = Request options
assist-profiles-options-help = Leave a field empty to use the provider’s default. The time limit covers the whole request, from connecting to the last word of the reply; the default is 60000 ms.
assist-profiles-max-tokens = Maximum reply tokens
assist-profiles-temperature = Temperature (0 to 2)
assist-profiles-timeout = Time limit (ms)
assist-profiles-save = Save profile
assist-profiles-assign-title = Who uses which profile
assist-profiles-assign-help = A value for a user wins over a value for their domain, which wins over the deployment default, unless a wider value is locked. A user assigned a profile that no longer exists has no AI assistant; their requests are not sent anywhere else.
assist-profiles-assign-unavailable = This server does not offer profile assignment yet.
assist-profiles-default = Deployment default
assist-profiles-default-none = None (use the endpoint above)
assist-profiles-default-lock = Locked: domains and users cannot use another profile
assist-profiles-default-save = Save default
assist-profiles-default-saved = The deployment default is saved.
assist-profiles-assign-ids = Profile ids for the tables below: { $ids }
assist-profiles-assign-label = AI provider profile
assist-profiles-by-domain = By domain
assist-profiles-by-user = By user
assist-profiles-grants-title = Local endpoints
assist-profiles-grants-help = A profile may reach a loopback or local network address (Ollama, LM Studio, vLLM) only when its exact host and port are listed here. Link-local and cloud metadata addresses can never be granted. Adding or removing one asks you to confirm it is you.
assist-profiles-grants-empty = No local endpoints are granted.
assist-profiles-grant-destination = Host and port
assist-profiles-grant-note = Note
assist-profiles-grant-add = Grant
assist-profiles-grant-remove = Remove { $destination }
assist-profiles-error-egress-refused = the address is not allowed (grant it under Local endpoints if it is yours)
assist-profiles-error-timeout = the time limit passed
assist-profiles-error-connect = no connection could be made
assist-profiles-error-unauthorized = the provider refused the API key
assist-profiles-error-not-found = the endpoint or model was not found
assist-profiles-error-rate-limited = the provider’s rate limit was reached
assist-profiles-error-rejected = the provider refused the request
assist-profiles-error-upstream = the provider reported an error
assist-profiles-error-bad-response = the answer was not in the expected format
assist-profiles-error-declined = the provider declined to answer
assist-profiles-error-unsupported = this profile cannot do that
