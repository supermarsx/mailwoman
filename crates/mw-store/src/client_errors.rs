//! Browser-side error reports (`client_error_group`, `client_error_event`).
//!
//! A report is grouped by its fingerprint (a hash of the error name and its top
//! normalised frames, computed by the browser and re-validated by the server).
//! A group keeps counts and the latest scrubbed name, message, boundary and route
//! template; an event keeps one occurrence's scrubbed stack and coarse client
//! class. No user id, address, token, URL query, message or file name reaches
//! this module; `account_hash` is a salted hash and is `None` unless the
//! administrator enabled it.
use crate::{Store, StoreError, q};
use serde::{Deserialize, Serialize};

/// Longest stored stack (bytes).
pub const MAX_STACK: usize = 12 * 1024;
/// Longest stored extra JSON (bytes).
pub const MAX_EXTRA: usize = 1024;
/// Most groups kept; the oldest-seen group is dropped past this.
pub const MAX_GROUPS: i64 = 5000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewClientError {
    pub id: String,
    pub fingerprint: String,
    pub kind: String,
    pub name: String,
    pub message: String,
    pub boundary: String,
    pub route_template: String,
    pub time: i64,
    pub build: String,
    pub occurrences: i64,
    pub visit: String,
    pub stack_raw: String,
    pub stack_symbolicated: Option<String>,
    pub ua_family: String,
    pub os_family: String,
    pub viewport: String,
    pub locale: String,
    pub account_hash: Option<String>,
    pub extra: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientErrorGroup {
    pub id: String,
    pub fingerprint: String,
    pub kind: String,
    pub name: String,
    pub message: String,
    pub boundary: String,
    pub route_template: String,
    pub first_seen: i64,
    pub last_seen: i64,
    pub count: i64,
    pub last_build: String,
    pub status: String,
    pub resolved_build: Option<String>,
    pub reopened_count: i64,
    /// Distinct page visits among the stored events (an estimate of affected sessions).
    pub visits: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientErrorEvent {
    pub id: String,
    pub group_id: String,
    pub time: i64,
    pub build: String,
    pub occurrences: i64,
    pub stack_raw: String,
    pub stack_symbolicated: Option<String>,
    pub ua_family: String,
    pub os_family: String,
    pub viewport: String,
    pub locale: String,
    pub account_hash: Option<String>,
    pub extra: String,
}

/// What recording one report did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    pub group_id: String,
    /// The group was created by this report.
    pub new_group: bool,
    /// A resolved group was reopened because a build that never saw it reported it.
    pub regressed: bool,
    /// The event row was dropped (the group is over its hourly sampling budget or
    /// is ignored); the counts were kept.
    pub sampled_out: bool,
}

/// Filters of [`Store::list_client_error_groups`].
#[derive(Debug, Clone, Default)]
pub struct GroupFilter {
    pub status: Option<String>,
    pub build: Option<String>,
    pub boundary: Option<String>,
    pub since: Option<i64>,
    pub limit: i64,
}

macro_rules! group_cols {
    () => {
        "g.id,g.fingerprint,g.kind,g.name,g.message,g.boundary,g.route_template,g.first_seen,g.last_seen,g.count,g.last_build,g.status,g.resolved_build,g.reopened_count,(SELECT COUNT(DISTINCT e.visit) FROM client_error_event e WHERE e.group_id=g.id AND e.visit<>'') AS visits"
    };
}
macro_rules! event_cols {
    () => {
        "id,group_id,time,build,occurrences,stack_raw,stack_symbolicated,ua_family,os_family,viewport,locale,account_hash,extra"
    };
}

fn group(r: &crate::backend::Row) -> ClientErrorGroup {
    ClientErrorGroup {
        id: r.get_string("id"),
        fingerprint: r.get_string("fingerprint"),
        kind: r.get_string("kind"),
        name: r.get_string("name"),
        message: r.get_string("message"),
        boundary: r.get_string("boundary"),
        route_template: r.get_string("route_template"),
        first_seen: r.get_i64("first_seen"),
        last_seen: r.get_i64("last_seen"),
        count: r.get_i64("count"),
        last_build: r.get_string("last_build"),
        status: r.get_string("status"),
        resolved_build: r.get_opt_string("resolved_build"),
        reopened_count: r.get_i64("reopened_count"),
        visits: r.get_i64("visits"),
    }
}

fn event(r: &crate::backend::Row) -> ClientErrorEvent {
    ClientErrorEvent {
        id: r.get_string("id"),
        group_id: r.get_string("group_id"),
        time: r.get_i64("time"),
        build: r.get_string("build"),
        occurrences: r.get_i64("occurrences"),
        stack_raw: r.get_string("stack_raw"),
        stack_symbolicated: r.get_opt_string("stack_symbolicated"),
        ua_family: r.get_string("ua_family"),
        os_family: r.get_string("os_family"),
        viewport: r.get_string("viewport"),
        locale: r.get_string("locale"),
        account_hash: r.get_opt_string("account_hash"),
        extra: r.get_string("extra"),
    }
}

impl Store {
    /// Record one report: find or create its group, count it, reopen a resolved
    /// group on a regression, and store the event unless the group is over
    /// `max_events_per_hour` (the counts are kept either way) or ignored.
    pub async fn record_client_error(
        &self,
        e: &NewClientError,
        max_events_per_hour: i64,
    ) -> Result<Recorded, StoreError> {
        if e.fingerprint.is_empty() || e.fingerprint.len() > 64 || e.id.is_empty() {
            return Err(StoreError::Corrupt("invalid client error".into()));
        }
        let occurrences = e.occurrences.clamp(1, 1_000_000);
        let existing = q("SELECT id,status,resolved_build,resolved_at FROM client_error_group WHERE fingerprint=?1")
            .bind(e.fingerprint.as_str())
            .fetch_optional(&self.backend)
            .await?;
        let mut group_id = uuid::Uuid::new_v4().to_string();
        let mut new_group = existing.is_none();
        let mut regressed = false;
        let mut ignored = false;
        if let Some(r) = &existing {
            group_id = r.get_string("id");
            let status = r.get_string("status");
            ignored = status == "ignored";
            if status == "resolved" {
                let resolved_at = r.get_opt_i64("resolved_at").unwrap_or(i64::MAX);
                let resolved_build = r.get_opt_string("resolved_build").unwrap_or_default();
                let seen_before = q("SELECT 1 AS one FROM client_error_event WHERE group_id=?1 AND build=?2 AND time<=?3 LIMIT 1")
                    .bind(group_id.as_str())
                    .bind(e.build.as_str())
                    .bind(resolved_at)
                    .fetch_optional(&self.backend)
                    .await?
                    .is_some();
                regressed = e.build != resolved_build && !seen_before;
            }
        } else {
            // A concurrent first report of the same fingerprint loses on the UNIQUE
            // index and is counted into the winner below.
            let inserted = q("INSERT INTO client_error_group(id,fingerprint,kind,name,message,boundary,route_template,first_seen,last_seen,count,last_build,status) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8,?9,?10,'open') ON CONFLICT(fingerprint) DO NOTHING")
                .bind(group_id.as_str())
                .bind(e.fingerprint.as_str())
                .bind(e.kind.as_str())
                .bind(e.name.as_str())
                .bind(e.message.as_str())
                .bind(e.boundary.as_str())
                .bind(e.route_template.as_str())
                .bind(e.time)
                .bind(occurrences)
                .bind(e.build.as_str())
                .execute(&self.backend)
                .await?;
            if inserted == 0 {
                new_group = false;
                let r = q("SELECT id FROM client_error_group WHERE fingerprint=?1")
                    .bind(e.fingerprint.as_str())
                    .fetch_one(&self.backend)
                    .await?;
                group_id = r.get_string("id");
            }
        }
        if existing.is_some() || !new_group {
            let sql = if regressed {
                "UPDATE client_error_group SET count=count+?2,last_seen=CASE WHEN last_seen>?3 THEN last_seen ELSE ?3 END,last_build=?4,message=?5,boundary=?6,route_template=?7,status='open',reopened_count=reopened_count+1 WHERE id=?1"
            } else {
                "UPDATE client_error_group SET count=count+?2,last_seen=CASE WHEN last_seen>?3 THEN last_seen ELSE ?3 END,last_build=?4,message=?5,boundary=?6,route_template=?7 WHERE id=?1"
            };
            q(sql)
                .bind(group_id.as_str())
                .bind(occurrences)
                .bind(e.time)
                .bind(e.build.as_str())
                .bind(e.message.as_str())
                .bind(e.boundary.as_str())
                .bind(e.route_template.as_str())
                .execute(&self.backend)
                .await?;
        }
        let recent = q("SELECT COUNT(*) AS n FROM client_error_event WHERE group_id=?1 AND time>=?2")
            .bind(group_id.as_str())
            .bind(e.time - 3600)
            .fetch_one(&self.backend)
            .await?
            .get_i64("n");
        let sampled_out = ignored || recent >= max_events_per_hour.max(1);
        if !sampled_out {
            let stack = truncate(&e.stack_raw, MAX_STACK);
            let sym = e.stack_symbolicated.as_deref().map(|s| truncate(s, MAX_STACK));
            let extra = if e.extra.len() > MAX_EXTRA { "{}".to_owned() } else { e.extra.clone() };
            q("INSERT INTO client_error_event(id,group_id,time,build,occurrences,visit,stack_raw,stack_symbolicated,ua_family,os_family,viewport,locale,account_hash,extra) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14) ON CONFLICT(id) DO NOTHING")
                .bind(e.id.as_str())
                .bind(group_id.as_str())
                .bind(e.time)
                .bind(e.build.as_str())
                .bind(occurrences)
                .bind(e.visit.as_str())
                .bind(stack)
                .bind(sym)
                .bind(e.ua_family.as_str())
                .bind(e.os_family.as_str())
                .bind(e.viewport.as_str())
                .bind(e.locale.as_str())
                .bind(e.account_hash.as_deref())
                .bind(extra)
                .execute(&self.backend)
                .await?;
        }
        Ok(Recorded { group_id, new_group, regressed, sampled_out })
    }

    pub async fn list_client_error_groups(
        &self,
        f: &GroupFilter,
    ) -> Result<Vec<ClientErrorGroup>, StoreError> {
        let rows = q(concat!("SELECT ", group_cols!(), " FROM client_error_group g WHERE (?1='' OR g.status=?1) AND (?2='' OR g.last_build=?2) AND (?3='' OR g.boundary=?3) AND g.last_seen>=?4 ORDER BY g.last_seen DESC LIMIT ?5"))
            .bind(f.status.as_deref().unwrap_or(""))
            .bind(f.build.as_deref().unwrap_or(""))
            .bind(f.boundary.as_deref().unwrap_or(""))
            .bind(f.since.unwrap_or(0))
            .bind(f.limit.clamp(1, 1000))
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(group).collect())
    }

    pub async fn get_client_error_group(
        &self,
        id: &str,
    ) -> Result<Option<ClientErrorGroup>, StoreError> {
        let row = q(concat!("SELECT ", group_cols!(), " FROM client_error_group g WHERE g.id=?1"))
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(group))
    }

    pub async fn client_error_events(
        &self,
        group_id: &str,
        limit: i64,
    ) -> Result<Vec<ClientErrorEvent>, StoreError> {
        let rows = q(concat!("SELECT ", event_cols!(), " FROM client_error_event WHERE group_id=?1 ORDER BY time DESC LIMIT ?2"))
            .bind(group_id)
            .bind(limit.clamp(1, 500))
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(event).collect())
    }

    /// Per-hour occurrence counts of one group since `since`: `(hour_start, count)`.
    pub async fn client_error_series(
        &self,
        group_id: &str,
        since: i64,
    ) -> Result<Vec<(i64, i64)>, StoreError> {
        let rows = q("SELECT (time/3600)*3600 AS h, CAST(SUM(occurrences) AS BIGINT) AS n FROM client_error_event WHERE group_id=?1 AND time>=?2 GROUP BY (time/3600)*3600 ORDER BY h")
            .bind(group_id)
            .bind(since)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(|r| (r.get_i64("h"), r.get_i64("n"))).collect())
    }

    /// Counts of one dimension over a group's stored events. `dim` is one of
    /// `ua_family`, `os_family`, `build`, `viewport`, `locale`.
    pub async fn client_error_breakdown(
        &self,
        group_id: &str,
        dim: &str,
    ) -> Result<Vec<(String, i64)>, StoreError> {
        let sql = match dim {
            "ua_family" => "SELECT ua_family AS v, CAST(SUM(occurrences) AS BIGINT) AS n FROM client_error_event WHERE group_id=?1 GROUP BY ua_family ORDER BY n DESC LIMIT 20",
            "os_family" => "SELECT os_family AS v, CAST(SUM(occurrences) AS BIGINT) AS n FROM client_error_event WHERE group_id=?1 GROUP BY os_family ORDER BY n DESC LIMIT 20",
            "build" => "SELECT build AS v, CAST(SUM(occurrences) AS BIGINT) AS n FROM client_error_event WHERE group_id=?1 GROUP BY build ORDER BY n DESC LIMIT 20",
            "viewport" => "SELECT viewport AS v, CAST(SUM(occurrences) AS BIGINT) AS n FROM client_error_event WHERE group_id=?1 GROUP BY viewport ORDER BY n DESC LIMIT 20",
            "locale" => "SELECT locale AS v, CAST(SUM(occurrences) AS BIGINT) AS n FROM client_error_event WHERE group_id=?1 GROUP BY locale ORDER BY n DESC LIMIT 20",
            _ => return Err(StoreError::Corrupt("unknown dimension".into())),
        };
        let rows = q(sql).bind(group_id).fetch_all(&self.backend).await?;
        Ok(rows.iter().map(|r| (r.get_string("v"), r.get_i64("n"))).collect())
    }

    /// Set a group's status. Resolving remembers the build and time; reopening
    /// clears them. Returns whether the group exists.
    pub async fn set_client_error_status(
        &self,
        id: &str,
        status: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        let n = match status {
            "resolved" => q("UPDATE client_error_group SET status='resolved',resolved_build=last_build,resolved_at=?2 WHERE id=?1")
                .bind(id)
                .bind(now)
                .execute(&self.backend)
                .await?,
            "open" => q("UPDATE client_error_group SET status='open',resolved_build=NULL,resolved_at=NULL WHERE id=?1")
                .bind(id)
                .execute(&self.backend)
                .await?,
            "ignored" => q("UPDATE client_error_group SET status='ignored',resolved_build=NULL,resolved_at=NULL WHERE id=?1")
                .bind(id)
                .execute(&self.backend)
                .await?,
            _ => return Err(StoreError::Corrupt("unknown status".into())),
        };
        Ok(n > 0)
    }

    pub async fn delete_client_error_group(&self, id: &str) -> Result<bool, StoreError> {
        q("DELETE FROM client_error_event WHERE group_id=?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(q("DELETE FROM client_error_group WHERE id=?1")
            .bind(id)
            .execute(&self.backend)
            .await?
            > 0)
    }

    /// Delete events older than `event_cutoff` and groups not seen since
    /// `group_cutoff` (with their events); then trim past [`MAX_GROUPS`].
    /// Returns rows removed.
    pub async fn prune_client_errors(
        &self,
        event_cutoff: i64,
        group_cutoff: i64,
    ) -> Result<u64, StoreError> {
        let mut n = q("DELETE FROM client_error_event WHERE time<?1")
            .bind(event_cutoff)
            .execute(&self.backend)
            .await?;
        n += q("DELETE FROM client_error_event WHERE group_id IN (SELECT id FROM client_error_group WHERE last_seen<?1)")
            .bind(group_cutoff)
            .execute(&self.backend)
            .await?;
        n += q("DELETE FROM client_error_group WHERE last_seen<?1")
            .bind(group_cutoff)
            .execute(&self.backend)
            .await?;
        let total = q("SELECT COUNT(*) AS n FROM client_error_group")
            .fetch_one(&self.backend)
            .await?
            .get_i64("n");
        if total > MAX_GROUPS {
            let over = total - MAX_GROUPS;
            n += q("DELETE FROM client_error_event WHERE group_id IN (SELECT id FROM client_error_group ORDER BY last_seen ASC LIMIT ?1)")
                .bind(over)
                .execute(&self.backend)
                .await?;
            n += q("DELETE FROM client_error_group WHERE id IN (SELECT id FROM client_error_group ORDER BY last_seen ASC LIMIT ?1)")
                .bind(over)
                .execute(&self.backend)
                .await?;
        }
        Ok(n)
    }

    /// Open groups, for the `client_errors_open` gauge.
    pub async fn count_open_client_error_groups(&self) -> Result<i64, StoreError> {
        Ok(q("SELECT COUNT(*) AS n FROM client_error_group WHERE status='open'")
            .fetch_one(&self.backend)
            .await?
            .get_i64("n"))
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}
