# Mailwoman — scoped API-key / MCP-key management (source locale: en).
#
# SPEC §20.1/§20.3, plan §3 e8. Security-explanatory copy (scope descriptions,
# the shown-once reveal warning, and the unattended-send disclosure) is
# safety-critical — translate it faithfully, never weaken it.
#
# Message ids are kebab-case and module-prefixed (apikeys-*).

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = API keys
apikeys-heading = API keys
apikeys-intro = Create scoped keys for scripts and integrations. Each key is hashed at rest, shown once, and individually revocable. Grant the least scope that works.

apikeys-label = Label
apikeys-label-aria = Key label
apikeys-label-placeholder = e.g. backup script

apikeys-create = Create key
apikeys-saved = I have saved it

apikeys-reveal-warning = Copy this secret now — it is shown once and cannot be retrieved again.
apikeys-copy = Copy
apikeys-copy-aria = Copy secret to clipboard
apikeys-copied = Copied to clipboard

apikeys-error-need-label = give the key a label so you can recognise it later
apikeys-error-create = could not create the key
apikeys-error-revoke = could not revoke the key

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = Existing keys
apikeys-existing = Existing keys
apikeys-none = No keys yet.
apikeys-created = created { $date }
apikeys-last-used = last used { $date }
apikeys-never-used = never used
apikeys-revoked-at = revoked { $date }
apikeys-revoke = Revoke
apikeys-status-active = Active
apikeys-status-revoked = Revoked

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = Scope
apikeys-capabilities = Capabilities
apikeys-cap-read = Read
apikeys-cap-send = Send
apikeys-cap-delete = Delete
apikeys-surface = Surface
apikeys-surface-mail-label = Mail
apikeys-surface-pim-label = PIM (calendar / tasks / notes / contacts)
apikeys-accounts = Accounts
apikeys-all-accounts = All accounts
apikeys-folders = Folders
apikeys-all-folders = All folders
apikeys-folder-ids = Folder ids (comma-separated)
apikeys-folder-subset-aria = Folder subset
apikeys-constraints = Constraints
apikeys-expiry = Expiry (RFC 3339, empty = no expiry)
apikeys-expiry-aria = Expiry
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = Rate limit (requests/min, empty = unlimited)
apikeys-rate-limit-aria = Rate limit
apikeys-ip-allowlist = IP allowlist (CIDR/IP, comma-separated; empty = any)
apikeys-ip-allowlist-aria = IP allowlist
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = read
apikeys-verb-send = send
apikeys-verb-delete = delete
apikeys-summary-mail = mail
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = no verbs
apikeys-summary-nothing = nothing
apikeys-summary-none = none
apikeys-summary-verbs-on = { $verbs } on { $surfaces }
apikeys-summary-all-accounts = all accounts
apikeys-summary-accounts = accounts: { $ids }
apikeys-summary-all-folders = all folders
apikeys-summary-folders = folders: { $ids }
apikeys-summary-expires = expires { $date }
apikeys-summary-rate = { $n } req/min
apikeys-summary-ips = IPs: { $ips }
apikeys-summary-mcp-tools = MCP tools: { $tools }
apikeys-summary-unattended = UNATTENDED send

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = MCP keys
apikeys-mcp-heading = MCP keys
apikeys-mcp-intro = Grant an AI agent (over the Model Context Protocol) exactly the tools you choose. Mail bodies returned to the agent are labelled untrusted input. Each tool is granted individually.
apikeys-mcp-label-aria = MCP key label
apikeys-mcp-label-placeholder = e.g. assistant agent
apikeys-mcp-create = Create MCP key
apikeys-mcp-reveal-warning = Copy this secret now — it is shown once.
apikeys-mcp-error-need-label = give the MCP key a label
apikeys-mcp-error-need-tool = grant at least one tool
apikeys-mcp-error-create = could not create the MCP key

apikeys-tools = Tools
apikeys-tools-group = MCP tools
apikeys-outbox-suffix = (Outbox-gated)

apikeys-unattended-aria = Unattended send
apikeys-unattended-allow = Allow unattended send (bypass the Outbox — requires admin countersign)
apikeys-unattended-pending = Unattended send stays inactive until an administrator countersigns this key.
apikeys-unattended-disclosure = By default a granted send lands in your Outbox and waits for you to confirm it in the app — automation cannot send on its own. Unattended send REMOVES that human-in-the-loop step so this key can send mail without confirmation. It additionally requires an administrator to countersign the key. Grant it only to automation you fully trust; a compromised unattended-send key can send mail as you with no prompt.

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = Search mail
apikeys-tool-mail-search-desc = Search messages. Results are labelled untrusted input.
apikeys-tool-mail-read-label = Read mail
apikeys-tool-mail-read-desc = Read a message body. Bodies are labelled untrusted input.
apikeys-tool-folders-list-label = List folders
apikeys-tool-folders-list-desc = List mailbox folders.
apikeys-tool-drafts-create-label = Create drafts
apikeys-tool-drafts-create-desc = Create a draft message (never sent automatically).
apikeys-tool-mail-send-label = Send mail
apikeys-tool-mail-send-desc = Queue a message to the Outbox for in-app confirmation.
apikeys-tool-calendar-read-label = Read calendar
apikeys-tool-calendar-read-desc = Read calendar events.
apikeys-tool-calendar-propose-label = Propose events
apikeys-tool-calendar-propose-desc = Propose (not commit) calendar events.
apikeys-tool-tasks-read-label = Read tasks
apikeys-tool-tasks-read-desc = Read tasks.
apikeys-tool-tasks-write-label = Write tasks
apikeys-tool-tasks-write-desc = Create or update tasks.
apikeys-tool-contacts-read-label = Read contacts
apikeys-tool-contacts-read-desc = Read contacts.
