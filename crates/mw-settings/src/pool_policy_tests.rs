use super::*;

#[test]
fn database_pool_policy_is_global_typed_and_restart_required() {
    let def = lookup("database.pool").unwrap();
    assert_eq!(def.scopes, G);
    assert!(!def.user_visible());
    assert_eq!(
        def.kind.describe(),
        json!({"type":"databasePool","restartRequired":true})
    );
    let defaults = serde_json::to_value(mw_store::pool_policy::PoolPolicy::default()).unwrap();
    assert_eq!(def.default_value(), defaults);
    assert_eq!(validate(def, &defaults, "value").unwrap(), defaults);
    assert!(validate(def, &json!({"sqliteMaxConnections":null}), "value").is_ok());
    for invalid in [
        json!({"unknown":1}),
        json!({"sqliteMaxConnections":0}),
        json!({"postgresMaxConnections":4096}),
        json!({"acquireTimeoutMs":0}),
        json!({"idleTimeoutSeconds":0}),
        json!({"minimumConnections":4,"sqliteMaxConnections":2}),
        json!({"postgresConnectionBudget":10,"postgresReservedConnections":10}),
        json!({"postgresConnectionBudget":10,"postgresReservedConnections":9,"replicaCount":2}),
    ] {
        assert!(validate(def, &invalid, "value").is_err(), "{invalid}");
    }
}

#[test]
fn pool_environment_pin_is_an_object_and_refuses_stored_overrides() {
    let def = lookup("database.pool").unwrap();
    let pin = json!({"postgresMaxConnections":12,"replicaCount":2});
    let env = |key: &str| (key == "MW_DB_POOL").then(|| pin.to_string());
    let state = env_state(def, &env).unwrap();
    assert_eq!(state.precedence, Precedence::Pin);
    assert_eq!(state.value, Some(pin.clone()));
    let effective = resolve_loaded(def, def.key, &SettingSubject::default(), None, &[], &env);
    assert_eq!(effective.value, pin);
    assert_eq!(effective.source, SettingSource::EnvPin);
    assert!(effective.locked());
    assert_eq!(refuse_if_pinned(def, &env).unwrap_err().code, "pinnedByEnv");
}
