//! Response validators: line-by-line ports of the Node validators in
//! fixture-requests.mjs and representative-workload.mjs, with the JavaScript
//! semantics of `js.rs`. Each comment quotes the Node expression it implements.
use crate::js::{J, R, Throw, set_size};
use crate::jv::Node;

/// (method, tag) of each call of a batch, in request order.
pub type Calls = Vec<(String, String)>;

#[derive(Clone, Debug)]
pub enum Validator {
    /// calibration only: no validation, body not kept
    None,
    /// calibration only: the body must be JSON
    AnyJson,
    /// calibration only: `body=>body.ok` (truthy)
    OkTrue,
    Identity {
        owner: String,
    },
    JmapSingle {
        name: String,
        account: String,
    },
    EmailGet {
        account: String,
        ids: Vec<String>,
    },
    Refresh {
        owner: String,
        native: bool,
    },
    Paged {
        calls: Calls,
        account: String,
        page: usize,
        next: usize,
        known: Vec<String>,
    },
    Body {
        calls: Calls,
        account: String,
        id: String,
        expected: String,
        bytes: f64,
        blob: String,
    },
    Download {
        size: usize,
    },
    MarkRead {
        calls: Calls,
        account: String,
        id: String,
    },
    /// move-and-restore (from -> to -> from) and delete-to-trash-and-restore
    Move {
        calls: Calls,
        account: String,
        id: String,
        from: String,
        to: String,
    },
    Session {
        account: String,
    },
    ListBatch {
        calls: Calls,
        account: String,
        page: usize,
    },
    Changes {
        calls: Calls,
        account: String,
    },
    Preferences,
    Appearance {
        owner: String,
    },
    Activity,
    Toggle {
        calls: Calls,
        account: String,
        id: String,
        keyword: String,
        first: bool,
        second: bool,
    },
    Search {
        calls: Calls,
        account: String,
        planted: f64,
    },
    DraftSave {
        calls: Calls,
        account: String,
    },
    DraftDiscard {
        calls: Calls,
        account: String,
        id: String,
        drafts: String,
    },
}

/// What a passing response tells the engine (only draft-save has an effect).
pub enum Verdict {
    Reject,
    Accept,
    /// draft-save accepted; the created id joins the discard pool (`onCreated`)
    Created(String),
}

impl Validator {
    pub fn wants_bytes(&self) -> bool {
        matches!(self, Validator::Download { .. })
    }

    /// `validateBytes`
    pub fn check_bytes(&self, bytes: &[u8]) -> bool {
        match self {
            // bytes.length===size && bytes.equals(Buffer.alloc(size,'ABCD'))
            Validator::Download { size } => {
                bytes.len() == *size && bytes.iter().enumerate().all(|(i, b)| *b == b"ABCD"[i % 4])
            }
            _ => false,
        }
    }

    /// `validate(JSON.parse(text))`; a throw rejects.
    pub fn check(&self, value: &Node<'_>) -> Verdict {
        match self.eval(J::of(value)) {
            Ok(Some(id)) => Verdict::Created(id),
            Ok(None) => Verdict::Accept,
            Err(Throw) => Verdict::Reject,
        }
    }

    /// Ok(None) = accepted, Ok(Some(id)) = draft created, Err = rejected or thrown.
    fn eval(&self, v: J<'_>) -> R<Option<String>> {
        let ok = |b: bool| if b { Ok(None) } else { Err(Throw) };
        match self {
            // v=>v.accountId===(p.ownerAccountId??p.accountId)
            Validator::Identity { owner } => ok(v.get("accountId")?.eq_str(owner)),
            // value.methodResponses?.[0]?.[0]===name&&value.methodResponses?.[0]?.[1]?.accountId===p.accountId
            Validator::JmapSingle { name, account } => ok(single(v, name, account)?),
            // valid(value)&&Array.isArray(value.methodResponses[0][1].list)&&ids.every(id=>value.methodResponses[0][1].list.some(row=>row.id===id))
            Validator::EmailGet { account, ids } => {
                if !single(v, "Email/get", account)? {
                    return Err(Throw);
                }
                let list = v.get("methodResponses")?.idx(0)?.idx(1)?.get("list")?;
                let Some(items) = list.array() else {
                    return Err(Throw);
                };
                for id in ids {
                    if !some(items, |row| Ok(row.get("id")?.eq_str(id)))? {
                        return Err(Throw);
                    }
                }
                Ok(None)
            }
            // v=>v.accountId===owner&&v.upstream===native
            Validator::Refresh { owner, native } => ok(v.get("accountId")?.eq_str(owner)
                && matches!(v.get("upstream")?, J::V(Node::Bool(b)) if b == native)),
            // !!v&&typeof v==='object'&&typeof v.apiUrl==='string'&&!!v.accounts&&Object.hasOwn(v.accounts,p.accountId)
            Validator::Session { account } => ok(v.truthy()
                && v.typeof_object()
                && v.get("apiUrl")?.is_string()
                && v.get("accounts")?.truthy()
                && v.get("accounts")?.has_own(account)?),
            // v?.version===1&&!!v.preferences&&typeof v.preferences==='object'&&!Array.isArray(v.preferences)
            Validator::Preferences => {
                if !v.oget("version")?.eq_num(1.0) {
                    return Err(Throw);
                }
                let p = v.get("preferences")?;
                ok(p.truthy() && p.typeof_object() && !p.is_array())
            }
            // v?.ownerId===owner&&v.document?.version===1&&typeof v.compiledCss==='string'&&Array.isArray(v.lockedFields)
            Validator::Appearance { owner } => ok(v.oget("ownerId")?.eq_str(owner)
                && v.get("document")?.oget("version")?.eq_num(1.0)
                && v.get("compiledCss")?.is_string()
                && v.get("lockedFields")?.is_array()),
            // v?.ok===true
            Validator::Activity => ok(v.oget("ok")?.is_true()),
            Validator::Download { .. } | Validator::None => Err(Throw),
            Validator::AnyJson => Ok(None),
            Validator::OkTrue => ok(v.get("ok")?.truthy()),
            Validator::Paged {
                calls,
                account,
                page,
                next,
                known,
            } => {
                let rows = batch(v, calls, account)?;
                let (first, headers1, second, headers2) = (rows[0], rows[1], rows[2], rows[3]);
                let size = known.len() as f64;
                let has = |id: J<'_>| known.iter().any(|k| id.eq_str(k));
                // Array.isArray(first.ids)&&Array.isArray(second.ids)&&first.ids.length===pageSize&&second.ids.length===nextSize
                let (Some(a), Some(b)) = (first.get("ids")?.array(), second.get("ids")?.array())
                else {
                    return Err(Throw);
                };
                if a.len() != *page || b.len() != *next {
                    return Err(Throw);
                }
                // &&first.total===known.size&&second.total===known.size&&first.position===0&&second.position===pageSize
                if !(first.get("total")?.eq_num(size)
                    && second.get("total")?.eq_num(size)
                    && first.get("position")?.eq_num(0.0)
                    && second.get("position")?.eq_num(*page as f64))
                {
                    return Err(Throw);
                }
                // &&new Set([...first.ids,...second.ids]).size===pageSize+nextSize&&[...].every(id=>known.has(id))
                let all: Vec<J<'_>> = a.iter().chain(b.iter()).map(J::V).collect();
                if set_size(&all) != page + next || !all.iter().all(|id| has(*id)) {
                    return Err(Throw);
                }
                // &&owned(headers1,first.ids)&&owned(headers2,second.ids)
                ok(owned(headers1, &a.iter().map(J::V).collect::<Vec<_>>())?
                    && owned(headers2, &b.iter().map(J::V).collect::<Vec<_>>())?)
            }
            Validator::Body {
                calls,
                account,
                id,
                expected,
                bytes,
                blob,
            } => {
                let rows = batch(v, calls, account)?;
                let row = rows[0];
                // owned(row,[id])
                if !owned_str(row, id)? {
                    return Err(Throw);
                }
                let item = row.get("list")?.idx(0)?;
                // Object.values(row.list[0].bodyValues??{}).some(bodyMatches)
                let values = item.get("bodyValues")?;
                let parts = if values.nullish() {
                    Vec::new()
                } else {
                    values.values()?
                };
                let mut matched = false;
                for part in parts {
                    if body_matches(part, expected, *bytes)? {
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    return Err(Throw);
                }
                // row.list[0].attachments?.some(part=>part.blobId===p.attachmentBlobId&&part.name==='sample.txt')
                let attachments = item.get("attachments")?;
                if attachments.nullish() {
                    return Err(Throw);
                }
                let Some(list) = attachments.array() else {
                    return Err(Throw);
                }; // `.some` of a non-array throws
                ok(some(list, |part| {
                    Ok(part.get("blobId")?.eq_str(blob) && part.get("name")?.eq_str("sample.txt"))
                })?)
            }
            Validator::MarkRead { calls, account, id } => {
                let rows = batch(v, calls, account)?;
                let (set, read, restore, restored) = (rows[0], rows[1], rows[2], rows[3]);
                // changed(set)&&changed(restore)&&owned(read,[id])&&read.list[0].keywords?.$seen===true&&owned(restored,[id])&&!restored.list[0].keywords?.$seen
                ok(changed(set, id)?
                    && changed(restore, id)?
                    && owned_str(read, id)?
                    && read
                        .get("list")?
                        .idx(0)?
                        .get("keywords")?
                        .oget("$seen")?
                        .is_true()
                    && owned_str(restored, id)?
                    && !restored
                        .get("list")?
                        .idx(0)?
                        .get("keywords")?
                        .oget("$seen")?
                        .truthy())
            }
            Validator::Move {
                calls,
                account,
                id,
                from,
                to,
            } => {
                let rows = batch(v, calls, account)?;
                let (a, ab, b, bb) = (rows[0], rows[1], rows[2], rows[3]);
                // changed(move)&&inFolder(moved,to)&&changed(restore)&&inFolder(restored,from)
                ok(changed(a, id)?
                    && in_folder(ab, id, to, self.move_owned())?
                    && changed(b, id)?
                    && in_folder(bb, id, from, self.move_owned())?)
            }
            Validator::ListBatch {
                calls,
                account,
                page,
            } => {
                let rows = batch(v, calls, account)?;
                let (query, getr, unread) = (rows[0], rows[1], rows[2]);
                // Array.isArray(query.ids)&&query.ids.length===pageSize&&typeof query.queryState==='string'&&Array.isArray(get.list)&&get.list.length===pageSize
                let Some(ids) = query.get("ids")?.array() else {
                    return Err(Throw);
                };
                if ids.len() != *page || !query.get("queryState")?.is_string() {
                    return Err(Throw);
                }
                let Some(list) = getr.get("list")?.array() else {
                    return Err(Throw);
                };
                if list.len() != *page {
                    return Err(Throw);
                }
                // &&query.ids.every(id=>get.list.some(row=>row.id===id))
                for id in ids {
                    if !some(list, |row| Ok(row.get("id")?.strict_eq(J::V(id))))? {
                        return Err(Throw);
                    }
                }
                // &&Number.isSafeInteger(unread.total)&&unread.total>=0&&Array.isArray(unread.ids)&&unread.ids.length===0
                let total = unread.get("total")?;
                ok(total.safe_integer()
                    && total.num().unwrap() >= 0.0
                    && unread.get("ids")?.array().is_some_and(<[_]>::is_empty))
            }
            Validator::Changes { calls, account } => {
                let rows = batch(v, calls, account)?;
                let (base, queried, changed_row) = (rows[0], rows[1], rows[2]);
                let state = base.get("queryState")?;
                let empty = |row: J<'_>| row.array().is_some_and(<[_]>::is_empty);
                // typeof base.queryState==='string'&&queried.oldQueryState===base.queryState&&queried.newQueryState===base.queryState&&noChange(queried.removed)&&noChange(queried.added)
                if !(state.is_string()
                    && queried.get("oldQueryState")?.strict_eq(state)
                    && queried.get("newQueryState")?.strict_eq(state)
                    && empty(queried.get("removed")?)
                    && empty(queried.get("added")?))
                {
                    return Err(Throw);
                }
                // &&changed.oldState===base.queryState&&changed.newState===base.queryState&&noChange(created/updated/destroyed)&&changed.hasMoreChanges!==true
                ok(changed_row.get("oldState")?.strict_eq(state)
                    && changed_row.get("newState")?.strict_eq(state)
                    && empty(changed_row.get("created")?)
                    && empty(changed_row.get("updated")?)
                    && empty(changed_row.get("destroyed")?)
                    && !changed_row.get("hasMoreChanges")?.is_true())
            }
            Validator::Toggle {
                calls,
                account,
                id,
                keyword,
                first,
                second,
            } => {
                let rows = batch(v, calls, account)?;
                let (a, ab, b, bb) = (rows[0], rows[1], rows[2], rows[3]);
                // changed(a,id)&&changed(b,id)&&one(ab,id)&&one(bb,id)&&Boolean(ab.list[0].keywords?.[keyword])===(first===true)&&Boolean(bb...)===(second===true)
                ok(changed(a, id)?
                    && changed(b, id)?
                    && one(ab, id)?
                    && one(bb, id)?
                    && ab
                        .get("list")?
                        .idx(0)?
                        .get("keywords")?
                        .oget(keyword)?
                        .truthy()
                        == *first
                    && bb
                        .get("list")?
                        .idx(0)?
                        .get("keywords")?
                        .oget(keyword)?
                        .truthy()
                        == *second)
            }
            Validator::Search {
                calls,
                account,
                planted,
            } => {
                let rows = batch(v, calls, account)?;
                let (q, g) = (rows[0], rows[1]);
                // q.total===p.plantedQuarterly&&Array.isArray(q.ids)&&q.ids.length===Math.min(50,p.plantedQuarterly)&&Array.isArray(g.list)&&g.list.length===q.ids.length
                if !q.get("total")?.eq_num(*planted) {
                    return Err(Throw);
                }
                let Some(ids) = q.get("ids")?.array() else {
                    return Err(Throw);
                };
                if ids.len() as f64 != planted.min(50.0) {
                    return Err(Throw);
                }
                ok(g.get("list")?
                    .array()
                    .is_some_and(|list| list.len() == ids.len()))
            }
            Validator::DraftSave { calls, account } => {
                let rows = batch(v, calls, account)?;
                let row = rows[0];
                // const id=row.created?.d1?.id; if(typeof id!=='string'||!id||row.notCreated&&Object.keys(row.notCreated).length)return false
                let created = row.get("created")?.oget("d1")?.oget("id")?;
                let Some(id) = created.str().filter(|s| !s.is_empty()) else {
                    return Err(Throw);
                };
                let not = row.get("notCreated")?;
                if not.truthy() && not.keys_len()? > 0 {
                    return Err(Throw);
                }
                Ok(Some(id.to_owned()))
            }
            Validator::DraftDiscard {
                calls,
                account,
                id,
                drafts,
            } => {
                let rows = batch(v, calls, account)?;
                let (saved, gone) = (rows[0], rows[1]);
                // Array.isArray(saved.list)&&saved.list.length===1&&saved.list[0].mailboxIds?.[p.draftsId]===true&&saved.list[0].keywords?.$draft===true
                let Some(list) = saved.get("list")?.array() else {
                    return Err(Throw);
                };
                if list.len() != 1 {
                    return Err(Throw);
                }
                let item = J::V(&list[0]);
                if !(item.get("mailboxIds")?.oget(drafts)?.is_true()
                    && item.get("keywords")?.oget("$draft")?.is_true())
                {
                    return Err(Throw);
                }
                // &&Array.isArray(gone.destroyed)&&gone.destroyed.includes(id)&&(!gone.notDestroyed||Object.keys(gone.notDestroyed).length===0)
                let Some(destroyed) = gone.get("destroyed")?.array() else {
                    return Err(Throw);
                };
                if !destroyed.iter().any(|d| J::V(d).eq_str(id)) {
                    return Err(Throw);
                }
                let not = gone.get("notDestroyed")?;
                ok(!not.truthy() || not.keys_len()? == 0)
            }
        }
    }

    /// move-and-restore checks folders with `owned` (representative); trash with `one` (goal mix).
    fn move_owned(&self) -> bool {
        matches!(self, Validator::Move { calls, .. } if calls.first().is_some_and(|(_, tag)| tag == "move"))
    }
}

fn some(items: &[Node<'_>], mut f: impl FnMut(J<'_>) -> R<bool>) -> R<bool> {
    for item in items {
        if f(J::V(item))? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// value.methodResponses?.[0]?.[0]===name&&value.methodResponses?.[0]?.[1]?.accountId===account
fn single(v: J<'_>, name: &str, account: &str) -> R<bool> {
    let first = v.get("methodResponses")?.oidx(0)?;
    Ok(first.oidx(0)?.eq_str(name) && first.oidx(1)?.oget("accountId")?.eq_str(account))
}

/// const rows=value?.methodResponses; Array.isArray(rows)&&rows.length===calls.length&&rows.every((row,index)=>
///  row[0]===calls[index][0]&&row[2]===calls[index][2]&&row[1]?.accountId===p.accountId) -> rows.map(row=>row[1])
fn batch<'a>(v: J<'a>, calls: &Calls, account: &str) -> R<Vec<J<'a>>> {
    let rows = v.oget("methodResponses")?;
    let Some(items) = rows.array() else {
        return Err(Throw);
    };
    if items.len() != calls.len() {
        return Err(Throw);
    }
    let mut out = Vec::with_capacity(items.len());
    for (row, (method, tag)) in items.iter().zip(calls) {
        let row = J::V(row);
        if !(row.idx(0)?.eq_str(method)
            && row.idx(2)?.eq_str(tag)
            && row.idx(1)?.oget("accountId")?.eq_str(account))
        {
            return Err(Throw);
        }
        out.push(row.idx(1)?);
    }
    Ok(out)
}

/// owned(row,wanted): Array.isArray(row.list)&&row.list.length===wanted.length&&new Set(row.list.map(item=>item.id)).size===wanted.length
///  &&wanted.every(id=>row.list.some(item=>item.id===id))&&(!row.notFound||row.notFound.length===0)
fn owned(row: J<'_>, wanted: &[J<'_>]) -> R<bool> {
    let Some(list) = row.get("list")?.array() else {
        return Ok(false);
    };
    if list.len() != wanted.len() {
        return Ok(false);
    }
    let ids: Vec<J<'_>> = list
        .iter()
        .map(|item| J::V(item).get("id"))
        .collect::<R<_>>()?;
    if set_size(&ids) != wanted.len() {
        return Ok(false);
    }
    for id in wanted {
        if !some(list, |item| Ok(item.get("id")?.strict_eq(*id)))? {
            return Ok(false);
        }
    }
    let not = row.get("notFound")?;
    Ok(!not.truthy() || not.get("length")?.eq_num(0.0))
}
fn owned_str(row: J<'_>, id: &str) -> R<bool> {
    let wanted = Node::Str(id.into());
    owned(row, &[J::V(&wanted)])
}
/// changed(row): row.updated&&Object.hasOwn(row.updated,id)&&(!row.notUpdated||Object.keys(row.notUpdated).length===0)
fn changed(row: J<'_>, id: &str) -> R<bool> {
    let updated = row.get("updated")?;
    if !updated.truthy() || !updated.has_own(id)? {
        return Ok(false);
    }
    let not = row.get("notUpdated")?;
    Ok(!not.truthy() || not.keys_len()? == 0)
}
/// one(row,id): Array.isArray(row.list)&&row.list.length===1&&row.list[0].id===id&&(!row.notFound||row.notFound.length===0)
fn one(row: J<'_>, id: &str) -> R<bool> {
    let Some(list) = row.get("list")?.array() else {
        return Ok(false);
    };
    if list.len() != 1 || !J::V(&list[0]).get("id")?.eq_str(id) {
        return Ok(false);
    }
    let not = row.get("notFound")?;
    Ok(!not.truthy() || not.get("length")?.eq_num(0.0))
}
/// inFolder(row,folder): owned|one(row,id)&&row.list[0].mailboxIds?.[folder]===true&&Object.keys(row.list[0].mailboxIds).length===1
fn in_folder(row: J<'_>, id: &str, folder: &str, use_owned: bool) -> R<bool> {
    let base = if use_owned {
        owned_str(row, id)?
    } else {
        one(row, id)?
    };
    if !base {
        return Ok(false);
    }
    let boxes = row.get("list")?.idx(0)?.get("mailboxIds")?;
    Ok(boxes.oget(folder)?.is_true() && boxes.keys_len()? == 1)
}
/// bodyMatches(part)
fn body_matches(part: J<'_>, expected: &str, bytes: f64) -> R<bool> {
    // typeof part?.value!=='string'||part.isTruncated===true||part.isEncodingProblem===true -> false
    let value = part.oget("value")?;
    let text: String = match value {
        J::V(Node::Str(s)) => s.to_string(),
        J::C(c) => c.to_string(),
        _ => return Ok(false),
    };
    if part.get("isTruncated")?.is_true() || part.get("isEncodingProblem")?.is_true() {
        return Ok(false);
    }
    let trimmed = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(&text);
    Ok(trimmed == expected && trimmed.len() as f64 == bytes)
}
