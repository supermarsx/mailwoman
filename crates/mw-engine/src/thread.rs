//! Engine-side threading (plan §1.7) — the canonical **JWZ** algorithm
//! (Jamie Zawinski, <https://www.jwz.org/doc/threading.html>).
//!
//! Threading is computed here, the same way for IMAP, POP3, and Gmail, so a
//! server's own `THREAD`/`X-GM-THRID` is at most an accelerator, never the
//! source of truth.
//!
//! ## Two entry points
//!
//! * [`thread`] runs the **full** JWZ set algorithm over a complete message
//!   set: it builds the id-keyed container table, links parent/child via
//!   `References`/`In-Reply-To`, computes the root set, prunes empty containers,
//!   and gathers threads by normalized subject. It returns the resulting
//!   container forest and is exercised by the reference corpora tests. It is
//!   also the shape a future one-shot re-thread migration would call.
//!
//! * [`thread_root`] is the **incremental** entry point used at ingest. Full
//!   JWZ is a *set* algorithm, but ingest is *per-message*, so [`thread_root`]
//!   runs the JWZ **build/link/root** phases over the member set formed by the
//!   newly-arriving message plus the reply-chain siblings the store already
//!   knows about, then returns that message's stable **root Message-ID**.
//!
//! ### Why the incremental path deliberately skips prune + subject-gather
//! JWZ's empty-container prune is a *display* transform: it collapses a phantom
//! root (a referenced-but-never-seen original) with a single child up into that
//! child. That is exactly wrong for a *stable thread identity* — a lone reply
//! whose original has not arrived yet must still root on the (phantom) original
//! so that, when the original later ingests, the two converge on the same
//! thread id. Keeping the phantom root reproduces (and generalizes, via sibling
//! repair) the `References[0]` identity the store keys threads off. Likewise
//! subject-gather needs every candidate's subject in memory, which the
//! incremental sibling lookup (keyed off the `message_id` column, no re-thread
//! of history) does not carry — so it is a no-op there and is proven instead by
//! the full-set corpora tests.
//!
//! This is the **new-ingest-only** decision: JWZ is applied to newly-arriving
//! mail; historical `thread_id`s are not re-keyed (no backfill/migration).

use std::collections::HashMap;

use mw_mime::ParsedEnvelope;

/// A message as far as threading is concerned: its identity plus the header
/// fields JWZ links on. Mirrors [`ParsedEnvelope`] with the `Subject` added
/// (subject-gather needs it).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Message {
    /// `Message-ID` (angle brackets already stripped by `mw-mime`).
    pub message_id: Option<String>,
    /// `In-Reply-To` (first id, angle brackets stripped).
    pub in_reply_to: Option<String>,
    /// `References` chain in header order (angle brackets stripped).
    pub references: Vec<String>,
    /// `Subject`, for subject-gather. Threading-irrelevant when absent.
    pub subject: Option<String>,
}

impl Message {
    /// Build from a parsed envelope and the message's subject.
    pub fn from_envelope(env: &ParsedEnvelope, subject: Option<&str>) -> Self {
        Self {
            message_id: env.message_id.clone(),
            in_reply_to: env.in_reply_to.clone(),
            references: env.references.clone(),
            subject: subject.map(str::to_owned),
        }
    }

    /// A sibling stub the store can materialize cheaply for the incremental
    /// path: a known ancestor identified by its Message-ID whose JWZ root is
    /// already recorded. Modelling it as `id` with `references = [root]` lets a
    /// newly-arriving reply chain up to `root` even when its own `References`
    /// header is truncated.
    pub fn stub(message_id: String, root_message_id: String) -> Self {
        let references = if root_message_id == message_id {
            Vec::new()
        } else {
            vec![root_message_id]
        };
        Self {
            message_id: Some(message_id),
            in_reply_to: None,
            references,
            subject: None,
        }
    }

    /// The effective parent chain JWZ links on: `References` when present, else
    /// the single `In-Reply-To` id (RFC 5256 / JWZ fallback).
    fn effective_refs(&self) -> Vec<String> {
        if self.references.is_empty() {
            self.in_reply_to.iter().cloned().collect()
        } else {
            self.references.clone()
        }
    }
}

/// A node in the JWZ container forest returned by [`thread`]. `message_id` is
/// `None` for an empty (phantom or synthetic subject-merge) container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadNode {
    pub message_id: Option<String>,
    pub children: Vec<ThreadNode>,
}

/// The stable **root Message-ID** for a newly-arriving message, given the
/// reply-chain siblings the store already knows (see module docs). `None` only
/// when the message has no usable identity at all (no `References`, no
/// `In-Reply-To`, no `Message-ID`), in which case the caller keys the thread
/// off the stable id.
///
/// Deterministic: the same `(target, siblings)` set always yields the same id.
pub fn thread_root(target: &Message, siblings: &[Message]) -> Option<String> {
    // Degenerate: nothing to thread on — matches the historical `None` contract.
    if target.message_id.is_none() && target.references.is_empty() && target.in_reply_to.is_none() {
        return None;
    }
    // Member set: the target first (index 0), then the ancestor stubs.
    let mut members = Vec::with_capacity(1 + siblings.len());
    members.push(target.clone());
    members.extend_from_slice(siblings);

    let mut t = Threader::default();
    t.build(&members);
    // The target is member 0; walk to the top of its (unpruned) tree so a
    // phantom original survives as the identity.
    t.identity_root(0)
}

/// Full JWZ over a complete message set: build → prune empty containers →
/// subject-gather. Returns the container forest (deterministically ordered).
pub fn thread(messages: &[Message]) -> Vec<ThreadNode> {
    let mut t = Threader::default();
    t.build(messages);
    t.prune();
    t.subject_gather();
    t.forest()
}

/// One JWZ container: an id-table slot that may hold a message and links up to a
/// parent / down to children. Indices reference the [`Threader::arena`].
#[derive(Debug, Default)]
struct Container {
    /// The id-table key this container was created under. `Some` for real
    /// messages and phantom `References` ids; `None`/`\0`-prefixed for the
    /// synthetic containers minted for duplicates, id-less messages, and
    /// subject merges.
    id: Option<String>,
    /// Index into the message set, when a real message occupies this slot.
    msg: Option<usize>,
    parent: Option<usize>,
    children: Vec<usize>,
    /// Set during prune/subject-gather so a spliced-out container is ignored.
    removed: bool,
}

#[derive(Default)]
struct Threader {
    arena: Vec<Container>,
    table: HashMap<String, usize>,
    /// message index -> its container index.
    msg_container: Vec<usize>,
    /// Copied threading fields per message (subject/message_id), so container
    /// walks don't need the caller's slice.
    messages: Vec<Message>,
    roots: Vec<usize>,
}

impl Threader {
    fn get_or_create(&mut self, id: &str) -> usize {
        if let Some(&i) = self.table.get(id) {
            return i;
        }
        let i = self.arena.len();
        self.arena.push(Container {
            id: Some(id.to_owned()),
            ..Container::default()
        });
        self.table.insert(id.to_owned(), i);
        i
    }

    fn new_synthetic(&mut self) -> usize {
        let i = self.arena.len();
        self.arena.push(Container::default());
        i
    }

    /// Is `node` inside the subtree rooted at `root` (used as the link loop
    /// guard: linking `root -> node` would loop iff `root` is already under
    /// `node`).
    fn in_subtree(&self, node: usize, root: usize) -> bool {
        if node == root {
            return true;
        }
        self.arena[root]
            .children
            .iter()
            .any(|&k| self.in_subtree(node, k))
    }

    /// JWZ step 1.B: link `parent -> child` in `References` order, keeping any
    /// existing parent and never introducing a loop.
    fn link(&mut self, parent: usize, child: usize) {
        if parent == child || self.arena[child].parent.is_some() || self.in_subtree(parent, child) {
            return;
        }
        self.arena[child].parent = Some(parent);
        self.arena[parent].children.push(child);
    }

    /// JWZ step 1.C: re-parent `child` under `new_parent` authoritatively
    /// (`References` is definitive), unless that would loop.
    fn reparent(&mut self, child: usize, new_parent: usize) {
        if child == new_parent || self.in_subtree(new_parent, child) {
            return;
        }
        if let Some(old) = self.arena[child].parent {
            self.arena[old].children.retain(|&k| k != child);
        }
        self.arena[child].parent = Some(new_parent);
        self.arena[new_parent].children.push(child);
    }

    /// JWZ step 1: id-table + parent/child links. Then step 2/3: the root set.
    fn build(&mut self, messages: &[Message]) {
        self.messages = messages.to_vec();
        self.msg_container = vec![usize::MAX; messages.len()];

        for (mi, m) in messages.iter().enumerate() {
            // 1.A: the container for this message.
            let cid = match &m.message_id {
                Some(id) => {
                    let c = self.get_or_create(id);
                    if self.arena[c].msg.is_some() {
                        // Duplicate Message-ID: give the newcomer its own slot.
                        let dup = self.new_synthetic();
                        self.arena[dup].msg = Some(mi);
                        dup
                    } else {
                        self.arena[c].msg = Some(mi);
                        c
                    }
                }
                None => {
                    let c = self.new_synthetic();
                    self.arena[c].msg = Some(mi);
                    c
                }
            };

            // 1.B: link the References chain.
            let refs = m.effective_refs();
            let mut prev: Option<usize> = None;
            for r in &refs {
                let ci = self.get_or_create(r);
                if let Some(p) = prev {
                    self.link(p, ci);
                }
                prev = Some(ci);
            }
            // 1.C: this message's parent is the last References element.
            if let Some(last) = refs.last() {
                let lp = self.get_or_create(last);
                self.reparent(cid, lp);
            }
            self.msg_container[mi] = cid;
        }

        self.recompute_roots();
    }

    fn recompute_roots(&mut self) {
        let mut roots: Vec<usize> = (0..self.arena.len())
            .filter(|&i| {
                let c = &self.arena[i];
                !c.removed && c.parent.is_none() && (c.msg.is_some() || !c.children.is_empty())
            })
            .collect();
        roots.sort_by_key(|&a| self.sort_key(a));
        self.roots = roots;
    }

    // ---- identity (incremental path) ------------------------------------

    /// Walk `msg_idx`'s container to the top of its tree and return that root's
    /// stable identity Message-ID.
    fn identity_root(&self, msg_idx: usize) -> Option<String> {
        let mut c = self.msg_container[msg_idx];
        while let Some(p) = self.arena[c].parent {
            c = p;
        }
        self.container_identity(c)
    }

    fn container_identity(&self, c: usize) -> Option<String> {
        if let Some(mi) = self.arena[c].msg
            && let Some(id) = &self.messages[mi].message_id
        {
            return Some(id.clone());
        }
        // Phantom `References` container: its real id-table key.
        if let Some(id) = &self.arena[c].id
            && !id.starts_with('\0')
        {
            return Some(id.clone());
        }
        // Synthetic (subject-merge / id-less): deterministic min descendant id.
        self.min_descendant_mid(c)
    }

    fn min_descendant_mid(&self, c: usize) -> Option<String> {
        let mut best: Option<String> = None;
        self.collect_mids(c, &mut best);
        best
    }

    fn collect_mids(&self, c: usize, best: &mut Option<String>) {
        if let Some(mi) = self.arena[c].msg
            && let Some(id) = &self.messages[mi].message_id
            && best.as_deref().map(|b| id.as_str() < b).unwrap_or(true)
        {
            *best = Some(id.clone());
        }
        for k in self.arena[c].children.clone() {
            self.collect_mids(k, best);
        }
    }

    // ---- prune (JWZ step 4) ---------------------------------------------

    fn prune(&mut self) {
        let roots = self.roots.clone();
        let mut new_roots = Vec::new();
        for r in roots {
            new_roots.extend(self.prune_walk(r, true));
        }
        for &r in &new_roots {
            self.arena[r].parent = None;
        }
        new_roots.sort_by_key(|&a| self.sort_key(a));
        self.roots = new_roots;
    }

    /// Recursively prune `c`, returning the containers that should take its
    /// place in the parent's child list (itself, its promoted children, or
    /// nothing).
    fn prune_walk(&mut self, c: usize, at_root: bool) -> Vec<usize> {
        let kids = self.arena[c].children.clone();
        let mut new_children = Vec::new();
        for k in kids {
            new_children.extend(self.prune_walk(k, false));
        }
        for &k in &new_children {
            self.arena[k].parent = Some(c);
        }
        self.arena[c].children = new_children;

        let has_msg = self.arena[c].msg.is_some();
        let nchild = self.arena[c].children.len();

        if !has_msg && nchild == 0 {
            // 4.A: empty leaf — nuke.
            self.arena[c].removed = true;
            return Vec::new();
        }
        if !has_msg {
            // 4.B: empty with children. Keep it only as a real grouping node,
            // i.e. an at-root container with more than one child; otherwise
            // splice the children up.
            if at_root && nchild > 1 {
                return vec![c];
            }
            self.arena[c].removed = true;
            return self.arena[c].children.clone();
        }
        vec![c]
    }

    // ---- subject-gather (JWZ step 5) ------------------------------------

    fn subject_gather(&mut self) {
        // 5.A/5.B: build the subject table over the root set.
        let mut table: HashMap<String, usize> = HashMap::new();
        for &c in &self.roots.clone() {
            let (subj, is_reply) = match self.container_subject(c) {
                Some(s) => s,
                None => continue,
            };
            match table.get(&subj).copied() {
                None => {
                    table.insert(subj, c);
                }
                Some(existing) => {
                    let ex_empty = self.arena[existing].msg.is_none();
                    let c_empty = self.arena[c].msg.is_none();
                    let ex_reply = self
                        .container_subject(existing)
                        .map(|(_, r)| r)
                        .unwrap_or(false);
                    if (c_empty && !ex_empty) || (!is_reply && ex_reply) {
                        table.insert(subj, c);
                    }
                }
            }
        }

        // 5.C: merge each root into the table's representative for its subject.
        for c in self.roots.clone() {
            if self.arena[c].removed {
                continue;
            }
            let (subj, c_reply) = match self.container_subject(c) {
                Some(s) => s,
                None => continue,
            };
            let other = match table.get(&subj).copied() {
                Some(o) if o != c && !self.arena[o].removed => o,
                _ => continue,
            };
            let c_empty = self.arena[c].msg.is_none();
            let o_empty = self.arena[other].msg.is_none();
            let o_reply = self
                .container_subject(other)
                .map(|(_, r)| r)
                .unwrap_or(false);

            if c_empty && o_empty {
                self.move_children(c, other);
            } else if c_empty && !o_empty {
                self.adopt(c, other);
            } else if !c_empty && o_empty {
                self.adopt(other, c);
            } else if o_reply && !c_reply {
                self.adopt(c, other);
            } else if !o_reply && c_reply {
                self.adopt(other, c);
            } else {
                let e = self.new_synthetic();
                self.adopt(e, c);
                self.adopt(e, other);
                table.insert(subj, e);
            }
        }

        self.recompute_roots();
    }

    /// The subject a container threads on: its own message's, else its first
    /// child's message's. Returns the normalized key and whether it was a reply
    /// (`Re:`/`Fwd:`-prefixed).
    fn container_subject(&self, c: usize) -> Option<(String, bool)> {
        let raw = self.arena[c]
            .msg
            .and_then(|mi| self.messages[mi].subject.clone())
            .or_else(|| {
                self.arena[c].children.first().and_then(|&k| {
                    self.arena[k]
                        .msg
                        .and_then(|mi| self.messages[mi].subject.clone())
                })
            })?;
        let (key, is_reply) = normalize_subject(&raw);
        if key.is_empty() {
            None
        } else {
            Some((key, is_reply))
        }
    }

    /// Make `child` (currently a root) a child of `parent`.
    fn adopt(&mut self, parent: usize, child: usize) {
        if parent == child {
            return;
        }
        if let Some(old) = self.arena[child].parent {
            self.arena[old].children.retain(|&k| k != child);
        }
        self.arena[child].parent = Some(parent);
        self.arena[parent].children.push(child);
    }

    /// Splice `from`'s children onto `to` and drop `from`.
    fn move_children(&mut self, from: usize, to: usize) {
        let kids = std::mem::take(&mut self.arena[from].children);
        for k in kids {
            self.arena[k].parent = Some(to);
            self.arena[to].children.push(k);
        }
        self.arena[from].removed = true;
    }

    // ---- forest materialization -----------------------------------------

    /// A deterministic sort key so sibling ordering is stable across runs.
    fn sort_key(&self, c: usize) -> String {
        self.container_identity(c).unwrap_or_default()
    }

    fn forest(&self) -> Vec<ThreadNode> {
        let mut roots = self.roots.clone();
        roots.sort_by_key(|&a| self.sort_key(a));
        roots.iter().map(|&r| self.node(r)).collect()
    }

    fn node(&self, c: usize) -> ThreadNode {
        let mut kids = self.arena[c]
            .children
            .iter()
            .copied()
            .filter(|&k| !self.arena[k].removed)
            .collect::<Vec<_>>();
        kids.sort_by_key(|&a| self.sort_key(a));
        ThreadNode {
            message_id: self.arena[c]
                .msg
                .and_then(|mi| self.messages[mi].message_id.clone()),
            children: kids.iter().map(|&k| self.node(k)).collect(),
        }
    }
}

/// Strip leading `Re:`/`Fwd:`/`Fw:` (and localized `Aw:`/`Sv:`) reply/forward
/// prefixes, fold case, and collapse whitespace. Returns the comparison key and
/// whether any prefix was stripped (i.e. this looked like a reply/forward).
fn normalize_subject(subject: &str) -> (String, bool) {
    let mut s = subject.trim();
    let mut is_reply = false;
    loop {
        let stripped = strip_one_prefix(s);
        match stripped {
            Some(rest) => {
                is_reply = true;
                s = rest.trim_start();
            }
            None => break,
        }
    }
    let key = s
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    (key, is_reply)
}

/// Strip a single leading `re`/`fwd`/`fw`/`aw`/`sv` prefix (case-insensitive,
/// optional `[nn]`, then `:`). Returns the remainder if one matched.
fn strip_one_prefix(s: &str) -> Option<&str> {
    const PREFIXES: [&str; 5] = ["re", "fwd", "fw", "aw", "sv"];
    let lower = s.to_ascii_lowercase();
    for p in PREFIXES {
        if let Some(rest) = lower.strip_prefix(p) {
            // Optional "[nn]" between the keyword and the colon (e.g. "Re[2]:").
            let mut byte_off = p.len();
            let after_kw = rest.trim_start();
            let consumed_ws = rest.len() - after_kw.len();
            byte_off += consumed_ws;
            let after_bracket = if let Some(b) = after_kw.strip_prefix('[') {
                if let Some(close) = b.find(']') {
                    if b[..close].chars().all(|c| c.is_ascii_digit()) {
                        byte_off += 1 + close + 1;
                        &after_kw[close + 2..]
                    } else {
                        after_kw
                    }
                } else {
                    after_kw
                }
            } else {
                after_kw
            };
            let trimmed = after_bracket.trim_start();
            byte_off += after_bracket.len() - trimmed.len();
            if trimmed.starts_with(':') {
                byte_off += 1;
                return Some(&s[byte_off..]);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(mid: &str, irt: Option<&str>, refs: &[&str], subject: Option<&str>) -> Message {
        Message {
            message_id: Some(mid.to_string()),
            in_reply_to: irt.map(String::from),
            references: refs.iter().map(|s| s.to_string()).collect(),
            subject: subject.map(String::from),
        }
    }

    // ---- incremental identity (engine ingest path) ----------------------

    #[test]
    fn original_roots_on_itself() {
        let e = msg("root@x", None, &[], None);
        assert_eq!(thread_root(&e, &[]).as_deref(), Some("root@x"));
    }

    #[test]
    fn reply_roots_on_references_head() {
        let e = msg("reply2@x", Some("reply1@x"), &["root@x", "reply1@x"], None);
        assert_eq!(thread_root(&e, &[]).as_deref(), Some("root@x"));
    }

    #[test]
    fn reply_without_references_uses_in_reply_to() {
        let e = msg("reply@x", Some("root@x"), &[], None);
        assert_eq!(thread_root(&e, &[]).as_deref(), Some("root@x"));
    }

    #[test]
    fn degenerate_no_headers_has_no_root() {
        let e = Message::default();
        assert_eq!(thread_root(&e, &[]), None);
    }

    #[test]
    fn reply_before_original_converges_when_original_arrives() {
        // Reply A references O; O has not been ingested yet (no siblings).
        let a = msg("a@x", Some("o@x"), &["o@x"], None);
        let a_root = thread_root(&a, &[]).unwrap();
        // The original O later ingests with no references and no siblings.
        let o = msg("o@x", None, &[], None);
        let o_root = thread_root(&o, &[]).unwrap();
        assert_eq!(a_root, "o@x");
        assert_eq!(o_root, "o@x");
        assert_eq!(a_root, o_root, "reply and original must share a root id");
    }

    #[test]
    fn sibling_repairs_a_truncated_references_chain() {
        // C's own References only reaches B, but the store knows B is rooted at
        // the true original A. The sibling stub pulls C up onto A.
        let c = msg("c@x", Some("b@x"), &["b@x"], None);
        let sib = Message::stub("b@x".into(), "a@x".into());
        assert_eq!(thread_root(&c, &[sib]).as_deref(), Some("a@x"));
        // Without the sibling, C roots on its own visible head (B).
        assert_eq!(thread_root(&c, &[]).as_deref(), Some("b@x"));
    }

    #[test]
    fn identity_is_deterministic() {
        let m = msg("z@x", Some("y@x"), &["a@x", "y@x"], None);
        let sibs = [
            Message::stub("y@x".into(), "a@x".into()),
            Message::stub("a@x".into(), "a@x".into()),
        ];
        let r1 = thread_root(&m, &sibs);
        let r2 = thread_root(&m, &sibs);
        assert_eq!(r1, r2);
        assert_eq!(r1.as_deref(), Some("a@x"));
    }

    // ---- full JWZ (reference corpora) -----------------------------------

    /// A depth-first list of `(message_id, depth)` for shape assertions.
    fn flatten(nodes: &[ThreadNode]) -> Vec<(Option<String>, usize)> {
        fn walk(n: &ThreadNode, depth: usize, out: &mut Vec<(Option<String>, usize)>) {
            out.push((n.message_id.clone(), depth));
            for c in &n.children {
                walk(c, depth + 1, out);
            }
        }
        let mut out = Vec::new();
        for n in nodes {
            walk(n, 0, &mut out);
        }
        out
    }

    #[test]
    fn linear_thread_nests_by_references() {
        let set = [
            msg("o@x", None, &[], Some("Hello")),
            msg("a@x", Some("o@x"), &["o@x"], Some("Re: Hello")),
            msg("b@x", Some("a@x"), &["o@x", "a@x"], Some("Re: Hello")),
        ];
        let forest = thread(&set);
        assert_eq!(forest.len(), 1);
        assert_eq!(
            flatten(&forest),
            vec![
                (Some("o@x".into()), 0),
                (Some("a@x".into()), 1),
                (Some("b@x".into()), 2),
            ]
        );
    }

    #[test]
    fn missing_root_kept_as_empty_container_with_multiple_children() {
        // Two replies to a never-seen original: JWZ keeps the empty root so the
        // siblings stay grouped (step 4.B multi-child at root).
        let set = [
            msg("a@x", Some("o@x"), &["o@x"], Some("Re: Topic")),
            msg("b@x", Some("o@x"), &["o@x"], Some("Re: Topic")),
        ];
        let forest = thread(&set);
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].message_id, None, "root is the empty original");
        let kids: Vec<_> = forest[0]
            .children
            .iter()
            .map(|c| c.message_id.clone().unwrap())
            .collect();
        assert_eq!(kids, vec!["a@x".to_string(), "b@x".to_string()]);
    }

    #[test]
    fn single_orphan_reply_promotes_over_empty_root() {
        // One reply to a never-seen original: the empty root collapses (step
        // 4.B single child) and the reply becomes the root.
        let set = [msg("a@x", Some("o@x"), &["o@x"], Some("Re: Solo"))];
        let forest = thread(&set);
        assert_eq!(flatten(&forest), vec![(Some("a@x".into()), 0)]);
    }

    #[test]
    fn subject_gather_merges_threads_with_no_shared_ids() {
        // No References between them, but the same normalized subject: JWZ
        // subject-gather makes the reply a child of the non-reply original.
        let set = [
            msg("orig@x", None, &[], Some("Project plan")),
            msg("reply@x", None, &[], Some("Re: Project plan")),
        ];
        let forest = thread(&set);
        assert_eq!(forest.len(), 1);
        assert_eq!(
            flatten(&forest),
            vec![(Some("orig@x".into()), 0), (Some("reply@x".into()), 1)]
        );
    }

    #[test]
    fn subject_gather_two_non_replies_get_a_synthetic_parent() {
        // Two distinct non-reply messages sharing a subject: JWZ mints an empty
        // container and files both under it.
        let set = [
            msg("m1@x", None, &[], Some("Weekly sync")),
            msg("m2@x", None, &[], Some("Weekly sync")),
        ];
        let forest = thread(&set);
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].message_id, None);
        let kids: Vec<_> = forest[0]
            .children
            .iter()
            .map(|c| c.message_id.clone().unwrap())
            .collect();
        assert_eq!(kids, vec!["m1@x".to_string(), "m2@x".to_string()]);
    }

    #[test]
    fn in_reply_to_links_when_references_absent() {
        let set = [
            msg("o@x", None, &[], Some("Q")),
            msg("r@x", Some("o@x"), &[], Some("Re: Q")),
        ];
        let forest = thread(&set);
        assert_eq!(
            flatten(&forest),
            vec![(Some("o@x".into()), 0), (Some("r@x".into()), 1)]
        );
    }

    #[test]
    fn duplicate_message_ids_do_not_collapse() {
        let set = [
            msg("dup@x", None, &[], Some("Dup")),
            msg("dup@x", None, &[], Some("Dup")),
        ];
        let forest = thread(&set);
        let flat = flatten(&forest);
        let count = flat
            .iter()
            .filter(|(m, _)| m.as_deref() == Some("dup@x"))
            .count();
        assert_eq!(count, 2, "both duplicates survive as distinct nodes");
    }

    #[test]
    fn reference_loops_are_broken() {
        // A references B and B references A: JWZ's loop guard keeps the graph a
        // tree. It must still terminate and yield every message.
        let set = [
            msg("a@x", None, &["b@x"], Some("Loop")),
            msg("b@x", None, &["a@x"], Some("Loop")),
        ];
        let forest = thread(&set);
        let flat = flatten(&forest);
        assert!(flat.iter().any(|(m, _)| m.as_deref() == Some("a@x")));
        assert!(flat.iter().any(|(m, _)| m.as_deref() == Some("b@x")));
    }

    #[test]
    fn empty_and_degenerate_set() {
        assert!(thread(&[]).is_empty());
        // A message with no id and no headers still materializes as a lone node.
        let forest = thread(&[Message::default()]);
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].message_id, None);
    }

    #[test]
    fn full_thread_is_deterministic() {
        let set = [
            msg("b@x", Some("o@x"), &["o@x"], Some("Re: T")),
            msg("o@x", None, &[], Some("T")),
            msg("a@x", Some("o@x"), &["o@x"], Some("Re: T")),
        ];
        assert_eq!(thread(&set), thread(&set));
    }

    #[test]
    fn subject_normalization_strips_prefixes() {
        assert_eq!(normalize_subject("Re: Hello").0, "hello");
        assert_eq!(normalize_subject("RE: FWD: Hello").0, "hello");
        assert_eq!(normalize_subject("Re[2]: Hello").0, "hello");
        assert_eq!(normalize_subject("Fwd: Hello world").0, "hello world");
        assert_eq!(normalize_subject("Plain").0, "plain");
        assert!(!normalize_subject("Plain").1);
        assert!(normalize_subject("Re: x").1);
    }
}
