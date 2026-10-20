use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::{
    accounts, admin, api, compile, desktop, drive, packages, realtime, sharing, AppState,
};

pub fn build_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/setup", get(accounts::setup::setup_status).post(accounts::setup::run_setup))
        .route("/admin/users", get(admin::users::list_users).post(admin::users::create_user))
        .route("/admin/users/{id}", patch(admin::users::update_user).delete(admin::users::delete_user))
        .route("/admin/account-policy", get(accounts::policy::get_policy).put(accounts::policy::update_policy))
        .route("/admin/api-access", get(api::limits::overview))
        .route("/admin/api-access/defaults", put(api::limits::update_defaults))
        .route("/admin/api-access/users/{id}", put(api::limits::update_user_limits))
        .route("/admin/package-policy", get(packages::policy::get_policy).put(packages::policy::update_policy))
        .route("/admin/storage", get(accounts::storage::overview))
        .route("/admin/storage/default-quota", put(accounts::storage::update_default_quota))
        .route("/admin/storage/users/{id}", put(accounts::storage::update_user_quota))
        .route("/compile", post(compile::handlers::compile_handler))
        .route("/export/{format}", post(compile::handlers::export_handler))
        .route("/export/pandoc/{format}", post(compile::handlers::pandoc_export_handler))
        .route("/import/pandoc", post(compile::handlers::pandoc_import_handler))
        .route("/lsp/{id}", get(compile::handlers::lsp_handler))
        .route("/auth/register", post(accounts::auth::register))
        .route("/auth/login", post(accounts::auth::login))
        .route("/auth/logout", post(accounts::auth::logout))
        .route("/auth/me", get(accounts::auth::me).put(accounts::auth::update_profile))
        .route(
            "/auth/avatar",
            get(accounts::avatars::get_avatar)
                .put(accounts::avatars::upload_avatar)
                .delete(accounts::avatars::delete_avatar)
                .layer(DefaultBodyLimit::max(accounts::avatars::MAX_UPLOAD_BYTES)),
        )
        .route("/auth/storage", get(accounts::storage::own_storage))
        .route("/auth/policy", get(accounts::policy::public_policy))
        .route("/auth/change-password", put(accounts::auth::change_password))
        .route("/folders", get(drive::folders::list_folders).post(drive::folders::create_folder))
        .route("/folders/shared", get(drive::folders::list_shared_folders))
        .route("/transfers", get(sharing::ownership::list_transfers).post(sharing::ownership::request_transfer))
        .route("/transfers/{id}", delete(sharing::ownership::cancel_transfer))
        .route("/transfers/{id}/accept", post(sharing::ownership::accept_transfer))
        .route("/share-info/{kind}/{id}", get(sharing::ownership::share_info))
        .route("/folders/{id}/collaborators", get(sharing::members::list_folder_collaborators))
        .route("/folders/{id}/collaborators/{collab_id}", delete(sharing::members::remove_folder_collaborator))
        .route("/folders/{id}/invite", post(sharing::members::invite_folder_collaborator))
        .route("/folders/{id}", delete(drive::folders::delete_folder).patch(drive::folders::update_folder))
        .route("/folders/{id}/download", get(drive::downloads::download_folder))
        .route("/fonts", get(drive::files::list_fonts))
        .route("/files", get(drive::files::list_files).post(drive::files::upload_file_global))
        .route("/files/{id}", delete(drive::files::delete_file).patch(drive::files::update_file))
        .route("/files/{id}/data", get(drive::files::get_file_data))
        .route("/docs/shared", get(drive::documents::list_shared_documents))
        .route("/docs", get(drive::documents::list_documents).post(drive::documents::create_document))
        .route("/docs/{id}", get(drive::documents::get_document).delete(drive::documents::delete_document).patch(drive::documents::update_document))
        .route("/docs/{id}/files", get(drive::document_assets::list_assets).post(drive::document_assets::upload_asset))
        .route("/docs/{id}/files/{file_id}", get(drive::document_assets::get_asset_data))
        .route("/docs/{id}/files/share-unshared", post(drive::document_assets::attach_unshared_assets))
        .route("/docs/{id}/files/{file_id}/attach", post(drive::document_assets::attach_asset))
        .route("/docs/{id}/fonts", get(drive::document_assets::list_fonts))
        .route("/docs/{id}/collaborators", get(sharing::members::list_document_collaborators))
        .route("/docs/{id}/collaborators/{collab_id}", delete(sharing::members::remove_document_collaborator))
        .route("/docs/{id}/invite", post(sharing::members::invite_document_collaborator))
        .route("/docs/{id}/comments", get(drive::collaboration::get_comments).post(drive::collaboration::add_comment))
        .route("/docs/{id}/versions", get(drive::collaboration::get_versions).post(drive::collaboration::create_version))
        .route("/comments/{id}", patch(drive::collaboration::update_comment).delete(drive::collaboration::delete_comment))
        .route("/keys", get(api::keys::list_keys).post(api::keys::create_key))
        .route("/keys/usage", get(api::keys::get_aggregate_usage))
        .route("/keys/limits", get(api::limits::own_limits))
        .route("/keys/{id}", delete(api::keys::delete_key))
        .route("/keys/{id}/regenerate", post(api::keys::regenerate_key))
        .route("/projects/shared", get(drive::projects::list_shared_projects))
        .route("/projects", get(drive::projects::list_projects).post(drive::projects::create_project))
        .route("/projects/{id}", get(drive::projects::get_project).delete(drive::projects::delete_project).patch(drive::projects::update_project))
        .route("/projects/{id}/download", get(drive::downloads::download_project))
        .route("/projects/{id}/collaborators", get(sharing::members::list_project_collaborators))
        .route("/projects/{id}/collaborators/{collab_id}", delete(sharing::members::remove_project_collaborator))
        .route("/projects/{id}/invite", post(sharing::members::invite_project_collaborator))
        .route("/projects/{id}/files", get(drive::projects::list_project_files).post(drive::projects::create_project_file))
        .route("/projects/{id}/files/upload", post(drive::projects::upload_project_file))
        .route("/projects/{id}/files/{fid}", get(drive::projects::get_project_file).patch(drive::projects::update_project_file).delete(drive::projects::delete_project_file))
        .route("/packages", get(packages::registry::list_packages))
        .route("/packages/publish", post(packages::registry::publish_package))
        .route("/packages/policy", get(packages::policy::own_policy))
        .route("/packages/{name}", get(packages::registry::list_versions).delete(packages::registry::delete_package))
        .route("/devices", get(desktop::devices::list_devices))
        .route("/devices/{id}", delete(desktop::devices::revoke_device));

    let desktop_routes = Router::new()
        .route("/version", get(desktop::version_info))
        .route("/auth/login", post(desktop::session::login))
        .route("/auth/logout", post(desktop::session::logout))
        .route("/auth/me", get(desktop::session::me))
        .route("/projects", get(desktop::projects::list_projects).post(desktop::projects::create_project))
        .route("/projects/{id}", get(desktop::project_files::pull_project).delete(desktop::projects::delete_project).patch(desktop::projects::move_project))
        .route("/projects/{id}/manifest", get(desktop::project_files::get_manifest))
        .route("/folders", get(desktop::folders::list_folders).post(desktop::folders::create_folder))
        .route("/folders/{id}", patch(desktop::folders::rename_folder).delete(desktop::folders::delete_folder))
        .route("/folders/{id}/move", patch(desktop::folders::move_folder))
        .route("/documents", get(desktop::documents::list_documents).post(desktop::documents::create_document))
        .route("/documents/{id}", get(desktop::documents::pull_document).put(desktop::documents::push_document).delete(desktop::documents::delete_document).patch(desktop::documents::move_document))
        .route("/shared", get(desktop::documents::list_shared))
        .route("/files", get(desktop::files::list_account_files).post(desktop::files::upload_account_file))
        .route("/files/{id}", get(desktop::files::pull_account_file).patch(desktop::files::rename_account_file).delete(desktop::files::delete_account_file))
        .route("/files/{id}/move", patch(desktop::files::move_account_file))
        .route("/projects/{id}/file", get(desktop::project_files::pull_file).put(desktop::project_files::push_file).delete(desktop::project_files::delete_file))
        .route("/ws", get(desktop::devices::ws_handler));

    let v1_routes = Router::new()
        .route("/render", post(api::render::render_handler));

    let yjs_routes = Router::new()
        .route("/{id}", get(realtime::yjs::yjs_handler));

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "../build".to_string());

    Router::new()
        .nest("/api", api_routes.nest("/desktop", desktop_routes).layer(TraceLayer::new_for_http()))
        .nest("/v1", v1_routes.layer(TraceLayer::new_for_http()))
        .nest("/yjs", yjs_routes.layer(TraceLayer::new_for_http()))
        .fallback_service(ServeDir::new(&static_dir).fallback(ServeFile::new(format!("{}/index.html", static_dir))))
        .with_state(state)
}
