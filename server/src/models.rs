use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Folder {
    pub id: String,
    pub owner_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: String,
    pub owner_id: String,
    pub document_id: Option<String>,
    pub folder_id: Option<String>,
    pub name: String,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: String,
    pub owner_id: String,
    pub folder_id: Option<String>,
    pub title: String,
    #[serde(skip_serializing)]
    pub content: Option<Vec<u8>>,
    pub thumbnail_svg: Option<String>,
    pub public_role: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub effective_role: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub folder_id: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub folder_id: Option<String>,
    pub public_role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageStats {
    pub documents_size_bytes: i64,
    pub files_size_bytes: i64,
    pub total_size_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Collaborator {
    pub id: String,
    pub document_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Invitation {
    pub id: String,
    pub document_id: String,
    pub role: String,
    pub token: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: String,
    pub document_id: String,
    pub user_id: String,
    pub content: String,
    pub resolved: bool,
    pub created_at: String,
    pub author_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentVersion {
    pub id: String,
    pub document_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: String,
    #[sqlx(default)]
    pub author_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVersionRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteRequest {
    pub email: String,
    pub role: String,
}
