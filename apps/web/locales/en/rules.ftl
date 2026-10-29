# Rules (mail-filter) module (audit #1, SPEC §6.1/§10.5). English source catalog.

rules-title = Rules & filters
rules-intro = Manage mail filtering scripts and visual rules. Available features depend on this account and administrator settings.
rules-empty = No rules yet.
rules-unavailable = This mail server does not support mail rules.
rules-load-failed = The rules could not be loaded.
rules-new = New rule
rules-list-label = Mail rules
rules-enabled = Enabled
rules-delete-named = Delete rule { $name }

# Builder
rules-builder-title = Rule builder
rules-builder-hint = Select a rule to edit, or create a new one.
rules-name-label = Rule name
rules-match-legend = When to run
rules-match-all = Match all conditions
rules-match-any = Match any condition
rules-conditions-legend = Conditions
rules-actions-legend = Actions
rules-add-condition = Add condition
rules-add-action = Add action
rules-remove-condition = Remove condition
rules-remove-action = Remove action
rules-cond-type-label = Field
rules-cond-op-label = Operator
rules-cond-value-label = Match value
rules-act-type-label = Action
rules-act-value-label = Action value
rules-save = Save rule
rules-editing = Editing { $name }
rules-save-refused = The rule was not saved: { $reason }

# Condition fields and operators
rules-cond-from = From
rules-cond-to = To
rules-cond-subject = Subject
rules-cond-thread-unsupported = Thread { $op } { $value }: thread conditions are not supported, so the rule cannot be saved until this condition is removed.
rules-op-contains = contains
rules-op-is = is

# Actions
rules-act-move = Move to mailbox
rules-act-tag = Add tag
rules-act-archive = Archive
rules-act-suppress = Suppress notification
rules-act-stop = Stop processing

# Where it runs
rules-runs-server = Server (Sieve)
rules-runs-server-detail = Uploaded to the mail server and applied there.
rules-runs-engine = Engine
rules-runs-engine-detail = Applied when the engine syncs new inbox messages; replies and moves wait for the next sync.

# Panes
rules-panes-label = Rule tools
rules-tab-builder = Builder
rules-tab-raw = Raw Sieve
rules-tab-dryrun = Dry run

# Raw editor
rules-raw-label = Raw Sieve source
rules-raw-reset = Reset to generated
rules-lint-clean = No problems found.

# Dry run
rules-dryrun-title = Dry run
rules-dryrun-help = Enter a sample message to preview which rules would fire.
rules-dryrun-results = Dry-run results
rules-dryrun-none = No enabled rules to evaluate.
rules-dryrun-nomatch = No match
rules-dryrun-stopped = Not reached (a prior rule stopped processing)

rules-scripts-label = Sieve scripts
rules-script-active = Active
rules-script-new = New script
rules-script-export = Export script
rules-script-import = Import script
rules-script-too-large = The script exceeds the size limit.
rules-script-read-only = Your administrator has disabled raw script editing.
rules-script-validate = Validate script
rules-script-deactivate = Deactivate
rules-script-activate = Activate
rules-script-saved = Script saved.

rules-vacation-title = Vacation reply
rules-vacation-enabled = Enable automatic replies
rules-vacation-from = Start (UTC)
rules-vacation-to = End (UTC)
rules-vacation-body = Reply text
rules-vacation-saved = Vacation reply saved.
rules-vacation-saved-off = Saved. Automatic replies are off.
rules-vacation-runs-server = Your mail server sends this reply when mail arrives.
rules-vacation-runs-engine = Mailwoman sends this reply when it syncs new inbox mail, so a reply can wait until the next sync.
rules-vacation-unavailable = Vacation replies are not available for this account: its mail server does not offer them, or your administrator has turned them off.
rules-script-template = Script template
rules-forward-title = Forwarding
rules-forward-address = Forward to
rules-forward-copy = Keep a copy in this account
rules-forward-refused = Your administrator does not allow forwarding to this domain.
