-- Repair applied 0043 schemas without changing their migration checksum.
-- migrate-store copies identity bindings and sessions in one deferred transaction.
ALTER TABLE identity_binding
    ALTER CONSTRAINT identity_binding_principal_id_fkey DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE session_principal
    ALTER CONSTRAINT session_principal_session_id_fkey DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE session_principal
    ALTER CONSTRAINT session_principal_principal_id_fkey DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE session_principal
    ALTER CONSTRAINT session_principal_binding_id_fkey DEFERRABLE INITIALLY DEFERRED;
