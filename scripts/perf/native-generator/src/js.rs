//! The small part of JavaScript value semantics the Node validators rely on,
//! over a parsed `jv::Node`. A JavaScript validator that throws (for
//! example `null.id`) rejects the response; here that is `Err(Throw)`.
//!
//! Covered: property access with and without optional chaining (objects, arrays,
//! strings, primitives), `===`, truthiness, `Array.isArray`, `typeof`,
//! `Object.hasOwn` / `Object.keys` / `Object.values`, `Number.isSafeInteger`,
//! `new Set(...).size` (SameValueZero; objects by identity) and `.length`.
//! String indexing yields one-character strings by Unicode scalar, not by UTF-16
//! unit; no validator compares such a character with anything but a method name or
//! an id, which are longer, so the verdict cannot differ.
use crate::jv::Node;

#[derive(Debug)]
pub struct Throw;
pub type R<T> = Result<T, Throw>;

#[derive(Clone, Copy, Debug)]
pub enum J<'a> {
    /// `undefined`
    U,
    V(&'a Node<'a>),
    /// a number that is not stored in the document (`array.length`)
    N(f64),
    /// one character of a string (`"abc"[0]`)
    C(char),
}

fn utf16_len(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

fn canonical_index(key: &str) -> Option<usize> {
    if key.is_empty()
        || (key.len() > 1 && key.starts_with('0'))
        || !key.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    key.parse().ok()
}

impl<'a> J<'a> {
    pub fn of(value: &'a Node<'a>) -> Self {
        J::V(value)
    }
    pub fn nullish(self) -> bool {
        matches!(self, J::U | J::V(Node::Null))
    }
    /// `a.key`
    pub fn get(self, key: &str) -> R<J<'a>> {
        match self {
            J::U | J::V(Node::Null) => Err(Throw),
            J::V(Node::Obj(map)) => Ok(Node::field(map, key).map_or(J::U, J::V)),
            J::V(Node::Arr(items)) => {
                if key == "length" {
                    Ok(J::N(items.len() as f64))
                } else {
                    Ok(canonical_index(key)
                        .and_then(|i| items.get(i))
                        .map_or(J::U, J::V))
                }
            }
            J::V(Node::Str(s)) => {
                if key == "length" {
                    Ok(J::N(utf16_len(s) as f64))
                } else {
                    Ok(canonical_index(key)
                        .and_then(|i| s.chars().nth(i))
                        .map_or(J::U, J::C))
                }
            }
            J::C(_) => Ok(if key == "length" {
                J::N(1.0)
            } else if key == "0" {
                self
            } else {
                J::U
            }),
            _ => Ok(J::U),
        }
    }
    /// `a?.key`
    pub fn oget(self, key: &str) -> R<J<'a>> {
        if self.nullish() {
            Ok(J::U)
        } else {
            self.get(key)
        }
    }
    /// `a[i]`
    pub fn idx(self, index: usize) -> R<J<'a>> {
        match self {
            J::V(Node::Arr(items)) => Ok(items.get(index).map_or(J::U, J::V)),
            _ => self.get(&index.to_string()),
        }
    }
    /// `a?.[i]`
    pub fn oidx(self, index: usize) -> R<J<'a>> {
        if self.nullish() {
            Ok(J::U)
        } else {
            self.idx(index)
        }
    }
    pub fn truthy(self) -> bool {
        match self {
            J::U | J::V(Node::Null) => false,
            J::V(Node::Bool(b)) => *b,
            J::V(Node::Num(f)) => *f != 0.0 && !f.is_nan(),
            J::N(f) => f != 0.0 && !f.is_nan(),
            J::V(Node::Str(s)) => !s.is_empty(),
            J::C(_) | J::V(Node::Arr(_)) | J::V(Node::Obj(_)) => true,
        }
    }
    pub fn array(self) -> Option<&'a [Node<'a>]> {
        match self {
            J::V(Node::Arr(items)) => Some(items),
            _ => None,
        }
    }
    pub fn is_array(self) -> bool {
        self.array().is_some()
    }
    pub fn str(self) -> Option<&'a str> {
        match self {
            J::V(Node::Str(s)) => Some(s),
            _ => None,
        }
    }
    pub fn is_string(self) -> bool {
        matches!(self, J::V(Node::Str(_)) | J::C(_))
    }
    pub fn num(self) -> Option<f64> {
        match self {
            J::V(Node::Num(f)) => Some(*f),
            J::N(f) => Some(f),
            _ => None,
        }
    }
    /// `typeof a === 'object'` (true for null, arrays and objects)
    pub fn typeof_object(self) -> bool {
        matches!(self, J::V(Node::Null | Node::Arr(_) | Node::Obj(_)))
    }
    pub fn eq_str(self, s: &str) -> bool {
        match self {
            J::V(Node::Str(v)) => v == s,
            J::C(c) => {
                let mut chars = s.chars();
                chars.next() == Some(c) && chars.next().is_none()
            }
            _ => false,
        }
    }
    pub fn eq_num(self, n: f64) -> bool {
        self.num().is_some_and(|v| v == n)
    }
    pub fn is_true(self) -> bool {
        matches!(self, J::V(Node::Bool(true)))
    }
    /// `a === b`
    pub fn strict_eq(self, other: J<'_>) -> bool {
        match (self, other) {
            (J::U, J::U) => true,
            (J::V(Node::Null), J::V(Node::Null)) => true,
            (J::V(Node::Bool(a)), J::V(Node::Bool(b))) => a == b,
            (a, b) if a.num().is_some() && b.num().is_some() => a.num() == b.num(),
            (a, b) if a.is_string() && b.is_string() => match (a, b) {
                (J::V(Node::Str(x)), J::V(Node::Str(y))) => x == y,
                (J::C(x), J::C(y)) => x == y,
                (J::C(c), J::V(Node::Str(s))) | (J::V(Node::Str(s)), J::C(c)) => J::C(c).eq_str(s),
                _ => false,
            },
            (J::V(a @ (Node::Arr(_) | Node::Obj(_))), J::V(b)) => std::ptr::eq(a, b),
            _ => false,
        }
    }
    /// `Object.hasOwn(a, key)`
    pub fn has_own(self, key: &str) -> R<bool> {
        match self {
            J::U | J::V(Node::Null) => Err(Throw),
            J::V(Node::Obj(map)) => Ok(Node::field(map, key).is_some()),
            J::V(Node::Arr(items)) => {
                Ok(key == "length" || canonical_index(key).is_some_and(|i| i < items.len()))
            }
            J::V(Node::Str(s)) => {
                Ok(key == "length" || canonical_index(key).is_some_and(|i| i < s.chars().count()))
            }
            J::C(_) => Ok(key == "length" || key == "0"),
            _ => Ok(false),
        }
    }
    /// `Object.keys(a).length`
    pub fn keys_len(self) -> R<usize> {
        match self {
            J::U | J::V(Node::Null) => Err(Throw),
            J::V(Node::Obj(map)) => Ok(Node::unique(map).count()),
            J::V(Node::Arr(items)) => Ok(items.len()),
            J::V(Node::Str(s)) => Ok(utf16_len(s)),
            J::C(c) => Ok(c.len_utf16()),
            _ => Ok(0),
        }
    }
    /// `Object.values(a)`
    pub fn values(self) -> R<Vec<J<'a>>> {
        match self {
            J::U | J::V(Node::Null) => Err(Throw),
            J::V(Node::Obj(map)) => Ok(Node::unique(map).map(|(_, v)| J::V(v)).collect()),
            J::V(Node::Arr(items)) => Ok(items.iter().map(J::V).collect()),
            J::V(Node::Str(s)) => Ok(s.chars().map(J::C).collect()),
            J::C(c) => Ok(vec![J::C(c)]),
            _ => Ok(Vec::new()),
        }
    }
    pub fn safe_integer(self) -> bool {
        self.num().is_some_and(|f| {
            f.is_finite() && f.fract() == 0.0 && f.abs() <= 9_007_199_254_740_991.0
        })
    }
}

/// `new Set(values).size` with SameValueZero; arrays and objects are distinct by identity.
pub fn set_size(values: &[J<'_>]) -> usize {
    let mut seen: Vec<J<'_>> = Vec::with_capacity(values.len());
    for value in values {
        let same = |seen: &J<'_>| match (seen, value) {
            (a, b) if a.num().is_some() && b.num().is_some() => {
                let (x, y) = (a.num().unwrap(), b.num().unwrap());
                x == y || (x.is_nan() && y.is_nan())
            }
            (a, b) => a.strict_eq(*b),
        };
        if !seen.iter().any(same) {
            seen.push(*value);
        }
    }
    seen.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jv::parse;

    #[test]
    fn access_and_throw_rules() {
        let v = parse(r#"{"a": null, "b": [1, {"id": "x"}], "s": "abc", "n": 5}"#).unwrap();
        let j = J::of(&v);
        assert!(j.get("a").unwrap().get("x").is_err());
        assert!(j.get("a").unwrap().oget("x").unwrap().nullish());
        assert!(j.get("missing").unwrap().get("x").is_err());
        assert!(j.get("n").unwrap().get("x").unwrap().nullish());
        assert!(j.get("b").unwrap().get("length").unwrap().eq_num(2.0));
        assert!(j.get("s").unwrap().idx(0).unwrap().eq_str("a"));
        assert!(
            j.get("b")
                .unwrap()
                .idx(1)
                .unwrap()
                .get("id")
                .unwrap()
                .eq_str("x")
        );
        assert_eq!(j.get("s").unwrap().keys_len().unwrap(), 3);
        assert_eq!(J::of(&parse("true").unwrap()).keys_len().unwrap(), 0);
        assert!(J::of(&parse("1.0").unwrap()).safe_integer());
        assert!(!J::of(&parse("1.5").unwrap()).safe_integer());
        let ids = parse(r#"["a", "a", {}, {}, 1, 1.0]"#).unwrap();
        let items: Vec<J> = J::of(&ids).array().unwrap().iter().map(J::V).collect();
        assert_eq!(set_size(&items), 4);
    }
}
