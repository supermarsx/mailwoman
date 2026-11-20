use mw_engine::Engine;
use mw_search::{IndexDoc,WriterOptions};
use mw_store::{ServerKey,Store};

#[tokio::test]
async fn sealed_constructor_keeps_directory_store_key_and_failure_behavior(){
    let directory=std::env::temp_dir().join(format!("mw-engine-sealed-options-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let expected=IndexDoc{stable_id:"fixture".into(),account_id:"owned".into(),body:"private sealed engine body".into(),..Default::default()};
    {
        let engine=Engine::open_with_search_options(store.clone(),&directory,WriterOptions{indexing_workers:Some(1),..Default::default()}).unwrap();
        assert_eq!(engine.search_index_dir(),Some(directory.as_path()));
        assert_eq!(engine.search().writer_configuration().indexing_workers,1);
        engine.search().upsert(&expected).unwrap();
    }
    assert!(std::fs::read(directory.join("meta.json")).unwrap().starts_with(b"MWB1"));
    {
        let engine=Engine::open_with_search(store.clone(),&directory).unwrap();
        assert_eq!(engine.search_index_dir(),Some(directory.as_path()));
        assert_eq!(engine.search().fetch_doc("fixture").unwrap(),Some(expected));
    }
    let invalid=directory.join("invalid");
    let error=match Engine::open_with_search_options(store.clone(),&invalid,WriterOptions{indexing_workers:Some(4),..Default::default()}){
        Ok(_)=>panic!("invalid worker options unexpectedly accepted"),Err(error)=>error,
    };
    assert!(error.to_string().contains("open search index:"));
    assert!(!invalid.exists());
    store.close().await;
    std::fs::remove_dir_all(directory).unwrap();
}
