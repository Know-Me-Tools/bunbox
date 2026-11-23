use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const RECIPE: &str = include_str!("../../../runtime/recipes/unanswered-demo-v1.ts");
pub const MAX_STAGED_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_RESULT_BYTES: usize = 64 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Submit {
    pub request_key: String,
    pub recipe_sha256: String,
    pub input: Input,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub rows: Vec<Row>,
    pub cutoff: i64,
    pub coverage: Coverage,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub inbox: bool,
    pub sent: bool,
}

#[derive(PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Row {
    pub source_ref: String,
    pub thread_ref: Option<String>,
    pub timestamp: i64,
    pub direction: Direction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

#[derive(PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Inbound,
    Outbound,
    Auto,
    Draft,
    Unknown,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Finding {
    pub thread_ref: String,
    pub source_ref: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultRows {
    pub unanswered: Vec<Finding>,
    pub unassigned: Vec<String>,
    pub coverage: String,
    pub truncated: bool,
}

pub fn digest(bytes: impl AsRef<[u8]>) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes.as_ref()))
}

pub fn bounded_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.bytes().all(|b| b.is_ascii_graphic())
}

impl Input {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.rows.len() <= 2000 && self.cutoff.unsigned_abs() <= 9_007_199_254_740_991,
            "invalid_input"
        );
        let mut seen = HashMap::new();
        for row in &self.rows {
            ensure!(
                bounded_id(&row.source_ref)
                    && row.thread_ref.as_deref().is_none_or(bounded_id)
                    && row.timestamp.unsigned_abs() <= 9_007_199_254_740_991
                    && row.timestamp <= self.cutoff
                    && row.subject.as_ref().is_none_or(|s| s.len() <= 1024),
                "invalid_row"
            );
            if let Some(previous) = seen.insert(&row.source_ref, row) {
                ensure!(previous == row, "source_ref_conflict");
            }
        }
        ensure!(
            serde_json::to_vec(self)?.len() <= MAX_STAGED_BYTES,
            "input_limit"
        );
        Ok(())
    }

    pub fn check_result(&self, out: &ResultRows) -> Result<()> {
        let complete = self.coverage.inbox && self.coverage.sent;
        let sources: HashMap<_, _> = self
            .rows
            .iter()
            .map(|r| (r.source_ref.as_str(), r))
            .collect();
        ensure!(
            out.coverage == if complete { "complete" } else { "partial" }
                && out.unanswered.len() <= 20
                && out.unassigned.len() <= 20
                && (complete || out.unanswered.is_empty()),
            "invalid_result"
        );
        let mut threads = HashSet::new();
        let mut refs = HashSet::new();
        for f in &out.unanswered {
            ensure!(
                sources
                    .get(f.source_ref.as_str())
                    .is_some_and(|row| row.direction == Direction::Inbound
                        && row.thread_ref.as_ref() == Some(&f.thread_ref))
                    && threads.insert(&f.thread_ref)
                    && refs.insert(&f.source_ref),
                "invalid_result_ref"
            );
        }
        for id in &out.unassigned {
            ensure!(
                sources
                    .get(id.as_str())
                    .is_some_and(|r| r.direction == Direction::Inbound)
                    && refs.insert(id),
                "invalid_result_ref"
            );
        }
        Ok(())
    }
}
