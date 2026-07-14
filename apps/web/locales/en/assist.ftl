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
