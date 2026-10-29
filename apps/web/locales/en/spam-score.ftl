# Mailwoman — the reader's header-reported spam score chips and their breakdown
# popover (SpamScore). Ids are kebab-case, prefixed `spam-score-`. The general
# widget strings (loading, unavailable, verdicts, the forgery caveat) stay in mail.ftl.

spam-score-chip-label = { $source } score { $score } of { $threshold }, { $band }. Show the rule breakdown.
spam-score-chip-label-no-threshold = { $source } score { $score }, { $band }. Show the rule breakdown.
spam-score-band-ham = likely not spam
spam-score-band-neutral = uncertain
spam-score-band-spam = likely spam
spam-score-band = Rating
spam-score-title = { $source } score breakdown
spam-score-total = Total
spam-score-threshold = Threshold
spam-score-threshold-missing = Not reported
spam-score-of-threshold = Threshold { $threshold }
spam-score-caption = Rules the header lists, largest contribution first.
spam-score-col-rule = Rule
spam-score-col-points = Points
spam-score-col-description = Description
spam-score-points-missing = Not given
spam-score-no-rules = The header lists no individual rules.
spam-score-not-scanned = Mailwoman shows these scores as the receiving server wrote them; it does not scan the message itself.
