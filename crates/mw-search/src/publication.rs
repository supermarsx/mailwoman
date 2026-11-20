//! Serialized durable publication primitive. Engine owns admission/source checks.
use crate::{Index, IndexDoc, Result, SearchError};
use std::sync::{Arc, Weak, atomic::Ordering};
use tantivy::{
    Term,
    query::{BooleanQuery, Occur, TermQuery},
    schema::IndexRecordOption,
};

#[derive(Clone)]
pub enum PublicationChange {
    Upsert(Arc<IndexDoc>),
    /// Delete only the indexed binding for this owner, never a reassigned ID.
    Delete {
        account_id: String,
        stable_id: String,
    },
}
impl std::fmt::Debug for PublicationChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Upsert(_) => "Upsert([redacted])",
            Self::Delete { .. } => "Delete([redacted])",
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationReceipt {
    pub opstamp: u64,
    pub searcher_generation: u64,
    pub changes: usize,
}
pub struct PublicationOwnerClaim {
    index: Weak<Index>,
}
impl Drop for PublicationOwnerClaim {
    fn drop(&mut self) {
        if let Some(index) = self.index.upgrade() {
            index.publication_owner.store(false, Ordering::Release);
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicationFault {
    ApplyAfter(usize),
    Commit,
    Reload,
    Rollback,
}

impl Index {
    pub fn claim_publication_owner(self: &Arc<Self>) -> Result<PublicationOwnerClaim> {
        self.publication_owner
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| SearchError::Io("index already has an active publication owner".into()))?;
        Ok(PublicationOwnerClaim {
            index: Arc::downgrade(self),
        })
    }
    /// Full index document only after an account+ID query. Source size is a
    /// fast-field check before stored document decoding; serialized bytes are
    /// capped before a second owned IndexDoc is materialized.
    pub fn publication_document(
        &self,
        account_id: &str,
        stable_id: &str,
        max_source_bytes: u64,
        max_json_bytes: usize,
    ) -> Result<Option<IndexDoc>> {
        use tantivy::schema::Value;
        let query = BooleanQuery::new(vec![
            (
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.fields.account_id, account_id),
                    IndexRecordOption::Basic,
                )) as Box<dyn tantivy::query::Query>,
            ),
            (
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.fields.stable_id, stable_id),
                    IndexRecordOption::Basic,
                )),
            ),
        ]);
        let searcher = self.reader.searcher();
        let hits = searcher.search(
            &query,
            &tantivy::collector::TopDocs::with_limit(2).order_by_score(),
        )?;
        if hits.len() > 1 {
            return Err(SearchError::Io(
                "ambiguous index publication binding".into(),
            ));
        }
        let Some((_, address)) = hits.first() else {
            return Ok(None);
        };
        let size = searcher
            .segment_reader(address.segment_ord)
            .fast_fields()
            .u64("size")?
            .first(address.doc_id)
            .unwrap_or(0);
        if size > max_source_bytes {
            return Err(SearchError::Io(
                "index source working-set limit exceeded".into(),
            ));
        }
        let stored: tantivy::schema::TantivyDocument = searcher.doc(*address)?;
        let Some(raw) = stored
            .get_first(self.fields.doc_json)
            .and_then(|value| value.as_str())
        else {
            return Ok(None);
        };
        if raw.len() > max_json_bytes {
            return Err(SearchError::Io(
                "index document working-set limit exceeded".into(),
            ));
        }
        let doc: IndexDoc = serde_json::from_str(raw)
            .map_err(|_| SearchError::Io("invalid stored publication document".into()))?;
        if doc.account_id != account_id || doc.stable_id != stable_id {
            return Err(SearchError::Io("stored publication binding differs".into()));
        }
        Ok(Some(doc))
    }

    pub fn publication_is_poisoned(&self) -> bool {
        self.publication_poisoned.load(Ordering::Acquire)
    }
    /// Used by the owner when optimistic source verification cannot converge.
    pub fn poison_publication(&self) {
        self.publication_poisoned.store(true, Ordering::Release);
    }

    /// Apply one bounded owner-prepared group, commit once, then reload while
    /// still holding the writer. Returned generation is actually query-visible.
    /// No owner/source authority is inferred from this low-level method.
    pub fn publish_batch(&self, changes: &[PublicationChange]) -> Result<PublicationReceipt> {
        if changes.is_empty() {
            return Err(SearchError::Io("empty index publication".into()));
        }
        let mut writer = self.lock_writer()?;
        #[cfg(test)]
        let fault = self.publication_fault.lock().unwrap().take();
        let apply = (|| -> Result<()> {
            for (position, change) in changes.iter().enumerate() {
                match change {
                    PublicationChange::Upsert(doc) => self.write_one(&writer, doc)?,
                    PublicationChange::Delete {
                        account_id,
                        stable_id,
                    } => {
                        writer.delete_query(Box::new(BooleanQuery::new(vec![
                            (
                                Occur::Must,
                                Box::new(TermQuery::new(
                                    Term::from_field_text(self.fields.account_id, account_id),
                                    IndexRecordOption::Basic,
                                )),
                            ),
                            (
                                Occur::Must,
                                Box::new(TermQuery::new(
                                    Term::from_field_text(self.fields.stable_id, stable_id),
                                    IndexRecordOption::Basic,
                                )),
                            ),
                        ])))?;
                    }
                }
                #[cfg(test)]
                if fault == Some(PublicationFault::ApplyAfter(position + 1))
                    || fault == Some(PublicationFault::Rollback)
                {
                    return Err(SearchError::Io("injected publication apply failure".into()));
                }
                #[cfg(not(test))]
                let _ = position;
            }
            Ok(())
        })();
        if let Err(error) = apply {
            #[cfg(test)]
            if fault == Some(PublicationFault::Rollback) {
                self.poison_publication();
                return Err(error);
            }
            if writer.rollback().is_err() {
                self.poison_publication();
            }
            return Err(error);
        }
        #[cfg(test)]
        if fault == Some(PublicationFault::Commit) {
            self.poison_publication();
            let _ = writer.rollback();
            return Err(SearchError::Io(
                "injected publication commit failure".into(),
            ));
        }
        let opstamp = match self.commit_writer_opstamp(&mut writer) {
            Ok(opstamp) => opstamp,
            Err(error) => {
                // A commit error may be durable already. Discard any remaining
                // staged buffer but never call that an authoritative rollback.
                self.poison_publication();
                let _ = writer.rollback();
                return Err(error);
            }
        };
        #[cfg(test)]
        if fault == Some(PublicationFault::Reload) {
            self.poison_publication();
            return Err(SearchError::Io(
                "injected publication reload failure after commit".into(),
            ));
        }
        if let Err(error) = self.reload_reader() {
            self.poison_publication();
            return Err(error);
        }
        Ok(PublicationReceipt {
            opstamp,
            searcher_generation: self.reader.searcher().generation().generation_id(),
            changes: changes.len(),
        })
    }

    /// Explicit owner recovery only. This discards uncommitted buffers and
    /// reloads durable index state; it does NOT repair source/index divergence.
    /// The Engine must retain affected dirty work and reconcile it before a
    /// successful source acknowledgement or reopening ordinary admission.
    pub fn recover_publication(&self) -> Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| SearchError::Io("index writer poisoned".into()))?;
        writer.rollback()?;
        self.reload_reader()?;
        self.publication_poisoned.store(false, Ordering::Release);
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn set_publication_fault(&self, fault: PublicationFault) {
        *self.publication_fault.lock().unwrap() = Some(fault);
    }
    #[cfg(test)]
    pub(crate) fn set_writer_lock_hook_for_test(&self, hook: Arc<std::sync::Barrier>) {
        *self.publication_lock_hook.lock().unwrap() = Some(hook);
    }
}
