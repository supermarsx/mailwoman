use mw_store::{ServerKey, Store};

#[tokio::test]
async fn vacation_reservation_is_atomic_expiring_and_account_bound() {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (first,second)=tokio::join!(
        store.claim_sieve_vacation("alice","same-sender-and-handle",100,60),
        store.claim_sieve_vacation("alice","same-sender-and-handle",100,60),
    );
    assert_ne!(first.unwrap(),second.unwrap());
    assert!(!store.claim_sieve_vacation("alice","same-sender-and-handle",159,60).await.unwrap());
    assert!(store.claim_sieve_vacation("alice","same-sender-and-handle",160,60).await.unwrap());
    assert!(store.claim_sieve_vacation("bob","same-sender-and-handle",100,60).await.unwrap());
}
