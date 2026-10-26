fn main() {
    // sqlx::migrate! embeds the inventory; newly added files must invalidate it too.
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=migrations_pg");
}
