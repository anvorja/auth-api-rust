// src/test/app_test.rs
use crate::app::mask_db_url;

#[test]
fn test_mask_db_url() {
    let url = "postgresql://user:password@localhost:5432/db";
    let masked = mask_db_url(url);
    assert_eq!(masked, "postgresql://user:****@localhost:5432/db");
}

#[test]
fn test_mask_db_url_no_password() {
    let url = "postgresql://localhost:5432/db";
    let masked = mask_db_url(url);
    assert_eq!(masked, "postgresql://localhost:5432/db");
}