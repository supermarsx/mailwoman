//! Linked preferences are an opt-in fallback below mailbox choices and admin policy.
pub(crate) fn should_inherit(group:Option<&str>,selected:&[&str],has_local_value:bool,locked:bool,is_fallback:bool)->bool{
    !has_local_value&&!locked&&is_fallback&&group.is_some_and(|group|selected.contains(&group))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn inheritance_is_opt_in_and_respects_local_and_administrator_precedence(){
  assert!(!should_inherit(Some("reader"),&[],false,false,true));
  assert!(should_inherit(Some("reader"),&["reader"],false,false,true));
  assert!(!should_inherit(Some("reader"),&["compose"],false,false,true));
  assert!(!should_inherit(None,&["reader"],false,false,true));
  assert!(!should_inherit(Some("reader"),&["reader"],true,false,true));
  assert!(!should_inherit(Some("reader"),&["reader"],false,true,true));
  // An unlocked domain or global administrator value still outranks inheritance.
  assert!(!should_inherit(Some("reader"),&["reader"],false,false,false));
 }
}
