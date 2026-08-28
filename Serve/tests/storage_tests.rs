use serve::domain::storage::BlobStorage;
use serve::infrastructure::storage::LocalStorage;

#[tokio::test]
async fn test_local_storage_upload_read_delete_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("ferro_test_{}", uuid::Uuid::new_v4()));
    let storage = LocalStorage::new(&temp_dir, "https://cdn.ferro.dev/media");

    let file_key = "avatars/user_123.png";
    let file_data = b"fake-png-image-bytes";

    // 1. Upload
    let url = storage
        .upload(file_key, "image/png", file_data)
        .await
        .expect("Upload should succeed");

    assert_eq!(url, "https://cdn.ferro.dev/media/avatars/user_123.png");

    // 2. Exists
    assert!(storage.exists(file_key).await);

    // 3. Read
    let read_bytes = storage.read(file_key).await.expect("Read should succeed");
    assert_eq!(read_bytes, file_data);

    // 4. Delete
    let deleted = storage
        .delete(file_key)
        .await
        .expect("Delete should succeed");
    assert!(deleted);

    // 5. Verify deleted
    assert!(!storage.exists(file_key).await);

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_local_storage_directory_traversal_prevention() {
    let temp_dir = std::env::temp_dir().join(format!("ferro_test_{}", uuid::Uuid::new_v4()));
    let storage = LocalStorage::new(&temp_dir, "/uploads");

    let malicious_key = "../../../etc/passwd";
    let res = storage.upload(malicious_key, "text/plain", b"hack").await;
    assert!(res.is_err());
}
