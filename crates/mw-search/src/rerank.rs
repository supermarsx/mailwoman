//! Cosine re-rank over an already-retrieved lexical hit list (SPEC §10.4/§14.3,
//! gap A8).
//!
//! ## Why this is not an ANN index
//! Retrieval stays entirely lexical: Tantivy's BM25 produces the candidate set
//! exactly as it always has, and this module only ever **permutes** a slice of
//! that set. Re-ranking a few hundred candidates is a few hundred dot products —
//! microseconds — so there is no vector index, no ANN crate, and no new
//! dependency. The cost of a real ANN structure is not worth paying for a feature
//! that is opt-in per query.
//!
//! ## The permutation rule
//! [`rerank_by_cosine`] moves **only** the hits that actually have a usable
//! vector, and moves them only among the positions those hits already collectively
//! occupy. A hit with no stored embedding, or one stored at a different
//! dimensionality, never moves and never gets displaced by more than the embedded
//! hits reshuffling around it. Two consequences that matter:
//!
//! * With zero usable vectors the output is **byte-identical** to the lexical
//!   input — the degradation path is the identity function, not a fallback that
//!   has to be kept in sync.
//! * With full coverage every position is occupied by an embedded hit, so the
//!   result is a straight cosine ordering of the whole slice.
//!
//! Ties (equal cosine, including the all-identical vectors a fixed-output stub
//! provider returns) preserve lexical order: the sort is stable and is performed
//! over positions taken in ascending order.

use std::collections::HashMap;

/// Cosine similarity of two equal-length vectors, or `None` when they disagree in
/// length, either is empty, either has a zero (or non-finite) norm, or the result
/// is not finite. `None` means "unrankable" — the caller leaves that hit where the
/// lexical search put it.
#[must_use]
pub fn cosine(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.is_empty() || a.len() != b.len() {
        return None;
    }
    // f64 accumulators: a 3072-wide dot product of f32s loses real precision, and
    // the whole point of this pass is the ORDER, which is decided by small
    // differences between neighbouring scores.
    let mut dot = 0.0_f64;
    let mut na = 0.0_f64;
    let mut nb = 0.0_f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let (x, y) = (f64::from(*x), f64::from(*y));
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    // A zero (or non-finite) vector has no direction to compare.
    if !(na.is_finite() && na > 0.0 && nb.is_finite() && nb > 0.0) {
        return None;
    }
    let sim = dot / (na.sqrt() * nb.sqrt());
    if sim.is_finite() {
        Some(sim as f32)
    } else {
        None
    }
}

/// Re-order `ids` in place by cosine similarity to `query`, using the vectors in
/// `vectors` (keyed by the same ids).
///
/// Only ids present in `vectors` with a vector of the same length as `query` take
/// part; every other id keeps its exact position. Returns the number of ids that
/// were actually scored — `0` means the slice was left untouched, which is the
/// documented degradation when a deployment's embedding model has changed
/// dimensionality out from under the stored vectors.
pub fn rerank_by_cosine(
    ids: &mut [String],
    query: &[f32],
    vectors: &HashMap<String, Vec<f32>>,
) -> usize {
    if ids.is_empty() || query.is_empty() {
        return 0;
    }

    // Positions that carry a usable vector, paired with their score. Collected in
    // ascending position order so the stable sort below breaks ties by lexical
    // rank.
    let mut scored: Vec<(usize, f32)> = Vec::new();
    for (pos, id) in ids.iter().enumerate() {
        if let Some(v) = vectors.get(id)
            && let Some(s) = cosine(query, v)
        {
            scored.push((pos, s));
        }
    }
    if scored.len() < 2 {
        return scored.len(); // nothing to permute (0 or 1 movable hit)
    }

    // The slots the embedded hits occupy, in ascending order — these are exactly
    // the positions we are allowed to write to.
    let slots: Vec<usize> = scored.iter().map(|(pos, _)| *pos).collect();
    // Sort the hits by descending score; `sort_by` is stable, so equal scores stay
    // in lexical order.
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));

    // Move the winners into the slots. Taking the ids out first keeps this a pure
    // permutation (no clone of the whole list, no id duplicated or dropped).
    let winners: Vec<String> = scored
        .iter()
        .map(|(from, _)| std::mem::take(&mut ids[*from]))
        .collect();
    for (slot, id) in slots.iter().zip(winners) {
        ids[*slot] = id;
    }
    slots.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    fn vecs(pairs: &[(&str, &[f32])]) -> HashMap<String, Vec<f32>> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.to_vec()))
            .collect()
    }

    #[test]
    fn cosine_basics() {
        assert_eq!(cosine(&[1.0, 0.0], &[1.0, 0.0]), Some(1.0));
        assert_eq!(cosine(&[1.0, 0.0], &[0.0, 1.0]), Some(0.0));
        assert_eq!(cosine(&[1.0, 0.0], &[-1.0, 0.0]), Some(-1.0));
        // Magnitude-invariant.
        assert_eq!(cosine(&[3.0, 0.0], &[7.0, 0.0]), Some(1.0));
        // Unrankable inputs.
        assert_eq!(cosine(&[1.0], &[1.0, 2.0]), None); // dimension mismatch
        assert_eq!(cosine(&[], &[]), None); // empty
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), None); // zero norm
    }

    #[test]
    fn reorders_by_similarity() {
        let mut list = ids(&["a", "b", "c"]);
        // Query points at `c`.
        let q = [1.0_f32, 0.0];
        let v = vecs(&[
            ("a", &[0.0, 1.0]), // orthogonal
            ("b", &[0.7, 0.7]), // partial
            ("c", &[1.0, 0.0]), // exact
        ]);
        assert_eq!(rerank_by_cosine(&mut list, &q, &v), 3);
        assert_eq!(list, ids(&["c", "b", "a"]));
    }

    #[test]
    fn no_vectors_leaves_the_list_byte_identical() {
        let original = ids(&["a", "b", "c", "d"]);
        let mut list = original.clone();
        assert_eq!(rerank_by_cosine(&mut list, &[1.0, 0.0], &HashMap::new()), 0);
        assert_eq!(list, original);
    }

    #[test]
    fn dimension_mismatch_degrades_to_lexical() {
        let original = ids(&["a", "b", "c"]);
        let mut list = original.clone();
        // Every stored vector is 3-wide; the query is 2-wide (the shape of a
        // deployment whose embedding model changed under a populated cache).
        let v = vecs(&[
            ("a", &[0.0, 1.0, 0.0]),
            ("b", &[1.0, 0.0, 0.0]),
            ("c", &[0.0, 0.0, 1.0]),
        ]);
        assert_eq!(rerank_by_cosine(&mut list, &[1.0, 0.0], &v), 0);
        assert_eq!(list, original, "not one hit moves on a dimension mismatch");
    }

    #[test]
    fn unembedded_hits_hold_their_positions() {
        // Only `a` and `d` have vectors, at positions 0 and 3. They may swap with
        // each other; `b` and `c` must not move at all.
        let mut list = ids(&["a", "b", "c", "d"]);
        let q = [1.0_f32, 0.0];
        let v = vecs(&[("a", &[0.0, 1.0]), ("d", &[1.0, 0.0])]);
        assert_eq!(rerank_by_cosine(&mut list, &q, &v), 2);
        assert_eq!(list, ids(&["d", "b", "c", "a"]));
    }

    #[test]
    fn ties_preserve_lexical_order() {
        // The shape a fixed-output stub provider produces: every document embeds
        // to the same vector, so every cosine is equal. Order must not churn.
        let original = ids(&["a", "b", "c", "d"]);
        let mut list = original.clone();
        let same: &[f32] = &[0.25, 0.5];
        let v = vecs(&[("a", same), ("b", same), ("c", same), ("d", same)]);
        assert_eq!(rerank_by_cosine(&mut list, &[1.0, 1.0], &v), 4);
        assert_eq!(list, original);
    }

    #[test]
    fn a_single_embedded_hit_never_moves() {
        let original = ids(&["a", "b", "c"]);
        let mut list = original.clone();
        let v = vecs(&[("b", &[1.0, 0.0])]);
        assert_eq!(rerank_by_cosine(&mut list, &[1.0, 0.0], &v), 1);
        assert_eq!(list, original);
    }

    #[test]
    fn permutation_is_lossless() {
        let mut list = ids(&["a", "b", "c", "d", "e"]);
        let q = [1.0_f32, 0.0, 0.0];
        let v = vecs(&[
            ("a", &[0.1, 0.9, 0.0]),
            ("b", &[0.9, 0.1, 0.0]),
            ("c", &[0.5, 0.5, 0.0]),
            ("d", &[0.0, 0.0, 1.0]),
            ("e", &[1.0, 0.0, 0.0]),
        ]);
        assert_eq!(rerank_by_cosine(&mut list, &q, &v), 5);
        let mut sorted = list.clone();
        sorted.sort();
        assert_eq!(sorted, ids(&["a", "b", "c", "d", "e"]));
        assert_eq!(list.first().map(String::as_str), Some("e"));
        assert!(!list.iter().any(String::is_empty), "no id was left taken");
    }

    #[test]
    fn empty_inputs_are_no_ops() {
        let mut empty: Vec<String> = Vec::new();
        assert_eq!(rerank_by_cosine(&mut empty, &[1.0], &HashMap::new()), 0);
        let mut list = ids(&["a"]);
        assert_eq!(rerank_by_cosine(&mut list, &[], &HashMap::new()), 0);
        assert_eq!(list, ids(&["a"]));
    }
}
