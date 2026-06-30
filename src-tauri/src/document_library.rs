use super::{database, reference_resolver};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

const HIGHLIGHT_ANNOTATION_SUBTYPE: i64 = 9;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Project {
    id: String,
    name: String,
    created_at: i64,
    updated_at: i64,
    last_opened_at: i64,
    document_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectStack {
    id: String,
    project_id: String,
    name: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectDocument {
    project_id: String,
    document: LibraryDocument,
    stack: ProjectStack,
    pile_id: Option<String>,
    position: i64,
    added_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LibraryDocument {
    id: String,
    content_hash: String,
    original_filename: String,
    title: String,
    byte_size: u64,
    stored_path: String,
    thumbnail_path: Option<String>,
    reference_id: Option<String>,
    reference_title: Option<String>,
    reference_authors: Vec<String>,
    created_at: i64,
    updated_at: i64,
    last_viewed_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentAnnotation {
    id: String,
    document_id: String,
    kind: String,
    page_index: u32,
    color: String,
    opacity: f64,
    selected_text: Option<String>,
    annotation: Value,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
struct LinkedReferenceData {
    title: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryChangedEvent {
    kind: &'static str,
    document_id: Option<String>,
    action: &'static str,
}

// Library mutations happen in whichever window is focused, but the data is shown
// in several windows at once (the organizer and any open viewers). After a
// committed change we broadcast "library-changed" so every window can reconcile
// from the database (the organizer refreshes its lists; a viewer closes when its
// own document is deleted, or refreshes its metadata when it is updated).
fn emit_library_changed(
    app: &AppHandle,
    kind: &'static str,
    document_id: Option<&str>,
    action: &'static str,
) {
    let _ = app.emit(
        "library-changed",
        LibraryChangedEvent {
            kind,
            document_id: document_id.map(ToOwned::to_owned),
            action,
        },
    );
}

#[tauri::command]
pub(crate) fn import_document(app: AppHandle, path: String) -> Result<LibraryDocument, String> {
    let source_path = Path::new(&path);
    let bytes = std::fs::read(source_path)
        .map_err(|error| format!("Could not read the PDF to import: {error}"))?;
    if !looks_like_pdf(&bytes) {
        return Err("The selected file is not a PDF".to_owned());
    }

    let content_hash = reference_resolver::document_digest(&bytes);
    let connection = database::connection(&app)?;
    if let Some(id) = document_id_by_hash(&connection, &content_hash)? {
        return load_document(&app, &connection, &id);
    }

    let id = Uuid::new_v4().to_string();
    let original_filename = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("document.pdf")
        .to_owned();
    let title = source_path
        .file_stem()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("Untitled document")
        .to_owned();
    let stored_path = document_path(&app, &id)?;
    let temporary_path = stored_path.with_extension("pdf.tmp");
    std::fs::write(&temporary_path, &bytes)
        .map_err(|error| format!("Could not copy the PDF into the library: {error}"))?;
    if let Err(error) = std::fs::rename(&temporary_path, &stored_path) {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(format!(
            "Could not finish copying the PDF into the library: {error}"
        ));
    }

    let now = database::unix_timestamp();
    let insert = connection.execute(
        r#"
        INSERT INTO documents (
            id, content_hash, original_filename, title, byte_size,
            created_at, updated_at, last_viewed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?6)
        "#,
        params![
            id,
            content_hash,
            original_filename,
            title,
            bytes.len() as i64,
            now
        ],
    );
    if let Err(error) = insert {
        let _ = std::fs::remove_file(&stored_path);
        if let Some(existing_id) = document_id_by_hash(&connection, &content_hash)? {
            return load_document(&app, &connection, &existing_id);
        }
        return Err(format!("Could not add the PDF to the library: {error}"));
    }
    // Render the first-page thumbnail (best-effort, pure Rust) before loading the
    // document, so the returned card already carries its thumbnailPath.
    let _ = crate::thumbnail::ensure_thumbnail(&app, &content_hash, &bytes);
    let document = load_document(&app, &connection, &id)?;
    emit_library_changed(&app, "document", Some(&id), "created");
    Ok(document)
}

#[tauri::command]
pub(crate) fn list_documents(app: AppHandle) -> Result<Vec<LibraryDocument>, String> {
    let connection = database::connection(&app)?;
    let mut statement = connection
        .prepare("SELECT id FROM documents ORDER BY last_viewed_at DESC, id")
        .map_err(|error| format!("Could not prepare the document list: {error}"))?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not list documents: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read the document list: {error}"))?;
    ids.iter()
        .map(|id| load_document(&app, &connection, id))
        .collect()
}

#[tauri::command]
pub(crate) fn get_document(app: AppHandle, id: String) -> Result<LibraryDocument, String> {
    load_document(&app, &database::connection(&app)?, &id)
}

#[tauri::command]
pub(crate) fn open_document(app: AppHandle, id: String) -> Result<LibraryDocument, String> {
    let connection = database::connection(&app)?;
    let changed = connection
        .execute(
            "UPDATE documents SET last_viewed_at = ?1 WHERE id = ?2",
            params![database::unix_timestamp(), id],
        )
        .map_err(|error| format!("Could not mark the document as viewed: {error}"))?;
    if changed == 0 {
        return Err("Document not found".to_owned());
    }
    let document = load_document(&app, &connection, &id)?;
    // Bumping last_viewed_at reorders the library, so let other windows know.
    emit_library_changed(&app, "document", Some(&id), "opened");
    Ok(document)
}

#[tauri::command]
pub(crate) fn rename_document(
    app: AppHandle,
    id: String,
    title: String,
) -> Result<LibraryDocument, String> {
    let title = clean_name(&title).ok_or_else(|| "Document title cannot be empty".to_owned())?;
    let connection = database::connection(&app)?;
    let changed = connection
        .execute(
            "UPDATE documents SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, database::unix_timestamp(), id],
        )
        .map_err(|error| format!("Could not rename the document: {error}"))?;
    if changed == 0 {
        return Err("Document not found".to_owned());
    }
    let document = load_document(&app, &connection, &id)?;
    emit_library_changed(&app, "document", Some(&id), "updated");
    Ok(document)
}

#[tauri::command]
pub(crate) fn delete_document(
    app: AppHandle,
    analysis: State<'_, crate::AnalysisManager>,
    id: String,
) -> Result<(), String> {
    let mut connection = database::connection(&app)?;
    let stored_path = document_path(&app, &id)?;
    let content_hash: Option<String> = connection
        .query_row(
            "SELECT content_hash FROM documents WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not look up the document: {error}"))?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start deleting the document: {error}"))?;
    let affected_projects = {
        let mut statement = transaction
            .prepare("SELECT project_id FROM project_documents WHERE document_id = ?1")
            .map_err(|error| format!("Could not inspect the document's projects: {error}"))?;
        let project_ids = statement
            .query_map(params![id], |row| row.get::<_, String>(0))
            .map_err(|error| format!("Could not inspect the document's projects: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read the document's projects: {error}"))?;
        project_ids
    };
    let changed = transaction
        .execute("DELETE FROM documents WHERE id = ?1", params![id])
        .map_err(|error| format!("Could not delete the document: {error}"))?;
    if changed == 0 {
        return Err("Document not found".to_owned());
    }
    for project_id in affected_projects {
        clear_singleton_piles(&transaction, &project_id)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not finish deleting the document: {error}"))?;
    // Stop (or forget) any background analysis for the document we just removed.
    analysis.cancel(&app, &id);
    emit_library_changed(&app, "document", Some(&id), "deleted");
    // Best-effort thumbnail cleanup; an orphan is harmless if this fails.
    if let Some(content_hash) = content_hash {
        if let Ok(thumbnail) = crate::thumbnail::thumbnail_path(&app, &content_hash) {
            let _ = std::fs::remove_file(thumbnail);
        }
    }
    match std::fs::remove_file(stored_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "The document record was deleted, but its PDF could not be removed: {error}"
        )),
    }
}

#[tauri::command]
pub(crate) fn create_project(app: AppHandle, name: String) -> Result<Project, String> {
    let name = clean_name(&name).ok_or_else(|| "Project name cannot be empty".to_owned())?;
    let id = Uuid::new_v4().to_string();
    let now = database::unix_timestamp();
    let connection = database::connection(&app)?;
    connection
        .execute(
            r#"
            INSERT INTO projects (id, name, name_key, created_at, updated_at, last_opened_at)
            VALUES (?1, ?2, ?3, ?4, ?4, ?4)
            "#,
            params![id, name, name_key(&name), now],
        )
        .map_err(|error| {
            format!("Could not create the project; its name may already exist: {error}")
        })?;
    load_project(&connection, &id)
}

#[tauri::command]
pub(crate) fn list_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let connection = database::connection(&app)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, name, created_at, updated_at, last_opened_at,
                   (SELECT COUNT(*) FROM project_documents pd WHERE pd.project_id = projects.id)
            FROM projects
            ORDER BY last_opened_at DESC, name_key, id
            "#,
        )
        .map_err(|error| format!("Could not prepare the project list: {error}"))?;
    let projects = statement
        .query_map([], row_to_project)
        .map_err(|error| format!("Could not list projects: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read the project list: {error}"))?;
    Ok(projects)
}

#[tauri::command]
pub(crate) fn get_project(app: AppHandle, id: String) -> Result<Project, String> {
    let connection = database::connection(&app)?;
    let changed = connection
        .execute(
            "UPDATE projects SET last_opened_at = ?1 WHERE id = ?2",
            params![database::unix_timestamp(), id],
        )
        .map_err(|error| format!("Could not mark the project as opened: {error}"))?;
    if changed == 0 {
        return Err("Project not found".to_owned());
    }
    load_project(&connection, &id)
}

#[tauri::command]
pub(crate) fn rename_project(app: AppHandle, id: String, name: String) -> Result<Project, String> {
    let name = clean_name(&name).ok_or_else(|| "Project name cannot be empty".to_owned())?;
    let connection = database::connection(&app)?;
    let changed = connection
        .execute(
            "UPDATE projects SET name = ?1, name_key = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, name_key(&name), database::unix_timestamp(), id],
        )
        .map_err(|error| {
            format!("Could not rename the project; its name may already exist: {error}")
        })?;
    if changed == 0 {
        return Err("Project not found".to_owned());
    }
    load_project(&connection, &id)
}

#[tauri::command]
pub(crate) fn delete_project(app: AppHandle, id: String) -> Result<(), String> {
    let connection = database::connection(&app)?;
    let changed = connection
        .execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|error| format!("Could not delete the project: {error}"))?;
    if changed == 0 {
        return Err("Project not found".to_owned());
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn create_project_stack(
    app: AppHandle,
    project_id: String,
    name: String,
) -> Result<ProjectStack, String> {
    let name = clean_name(&name).ok_or_else(|| "Stack name cannot be empty".to_owned())?;
    let id = Uuid::new_v4().to_string();
    let now = database::unix_timestamp();
    let connection = database::connection(&app)?;
    require_project(&connection, &project_id)?;
    connection
        .execute(
            r#"
            INSERT INTO project_stacks (
                id, project_id, name, name_key, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
            "#,
            params![id, project_id, name, name_key(&name), now],
        )
        .map_err(|error| {
            format!("Could not create the project stack; its name may already exist: {error}")
        })?;
    load_project_stack(&connection, &project_id, &id)
}

#[tauri::command]
pub(crate) fn list_project_stacks(
    app: AppHandle,
    project_id: String,
) -> Result<Vec<ProjectStack>, String> {
    let connection = database::connection(&app)?;
    require_project(&connection, &project_id)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM project_stacks
            WHERE project_id = ?1
            ORDER BY name_key, id
            "#,
        )
        .map_err(|error| format!("Could not prepare the project stack list: {error}"))?;
    let stacks = statement
        .query_map(params![project_id], row_to_project_stack)
        .map_err(|error| format!("Could not list project stacks: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read the project stack list: {error}"))?;
    Ok(stacks)
}

#[tauri::command]
pub(crate) fn rename_project_stack(
    app: AppHandle,
    project_id: String,
    stack_id: String,
    name: String,
) -> Result<ProjectStack, String> {
    let name = clean_name(&name).ok_or_else(|| "Stack name cannot be empty".to_owned())?;
    let connection = database::connection(&app)?;
    let changed = connection
        .execute(
            r#"
            UPDATE project_stacks
            SET name = ?1, name_key = ?2, updated_at = ?3
            WHERE project_id = ?4 AND id = ?5
            "#,
            params![
                name,
                name_key(&name),
                database::unix_timestamp(),
                project_id,
                stack_id
            ],
        )
        .map_err(|error| {
            format!("Could not rename the project stack; its name may already exist: {error}")
        })?;
    if changed == 0 {
        return Err("Project stack not found".to_owned());
    }
    load_project_stack(&connection, &project_id, &stack_id)
}

#[tauri::command]
pub(crate) fn delete_project_stack(
    app: AppHandle,
    project_id: String,
    stack_id: String,
) -> Result<(), String> {
    let connection = database::connection(&app)?;
    let document_count: i64 = connection
        .query_row(
            r#"
            SELECT COUNT(*)
            FROM project_documents
            WHERE project_id = ?1 AND stack_id = ?2
            "#,
            params![project_id, stack_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not check project stack documents: {error}"))?;
    if document_count > 0 {
        return Err("Project stack still contains documents".to_owned());
    }
    let changed = connection
        .execute(
            "DELETE FROM project_stacks WHERE project_id = ?1 AND id = ?2",
            params![project_id, stack_id],
        )
        .map_err(|error| format!("Could not delete the project stack: {error}"))?;
    if changed == 0 {
        return Err("Project stack not found".to_owned());
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn list_project_documents(
    app: AppHandle,
    project_id: String,
) -> Result<Vec<ProjectDocument>, String> {
    let connection = database::connection(&app)?;
    require_project(&connection, &project_id)?;
    load_project_documents(&app, &connection, &project_id)
}

fn load_project_documents(
    app: &AppHandle,
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectDocument>, String> {
    let mut statement = connection
        .prepare(
            r#"
            SELECT pd.document_id
            FROM project_documents pd
            JOIN project_stacks ps ON ps.project_id = pd.project_id AND ps.id = pd.stack_id
            JOIN documents d ON d.id = pd.document_id
            WHERE pd.project_id = ?1
            ORDER BY ps.name_key, pd.position, d.title, d.id
            "#,
        )
        .map_err(|error| format!("Could not prepare project documents: {error}"))?;
    let document_ids = statement
        .query_map(params![project_id], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not list project documents: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read project documents: {error}"))?;
    document_ids
        .iter()
        .map(|document_id| load_project_document(app, connection, project_id, document_id))
        .collect()
}

#[tauri::command]
pub(crate) fn add_document_to_project(
    app: AppHandle,
    project_id: String,
    document_id: String,
    stack_id: String,
) -> Result<ProjectDocument, String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start adding the document to the project: {error}"))?;
    require_document(&transaction, &document_id)?;
    require_project_stack(&transaction, &project_id, &stack_id)?;
    let now = database::unix_timestamp();
    let added_at = transaction
        .query_row(
            r#"
            SELECT added_at
            FROM project_documents
            WHERE project_id = ?1 AND document_id = ?2
            "#,
            params![project_id, document_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| format!("Could not check project document membership: {error}"))?
        .unwrap_or(now);
    let next_position: i64 = transaction
        .query_row(
            r#"
            SELECT COALESCE(MAX(position) + 1, 0)
            FROM project_documents
            WHERE project_id = ?1 AND stack_id = ?2
            "#,
            params![project_id, stack_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not determine the stack position: {error}"))?;
    transaction
        .execute(
            r#"
            INSERT INTO project_documents (
                project_id, document_id, stack_id, position, added_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(project_id, document_id) DO UPDATE SET
                stack_id = excluded.stack_id,
                updated_at = excluded.updated_at
            "#,
            params![project_id, document_id, stack_id, next_position, added_at, now],
        )
        .map_err(|error| format!("Could not add the document to the project: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not finish adding the document to the project: {error}"))?;
    let project_document = load_project_document(&app, &connection, &project_id, &document_id)?;
    emit_library_changed(&app, "projectDocument", Some(&document_id), "added");
    Ok(project_document)
}

/// Rewrites the stack membership and ordering for one column. The frontend sends
/// the full, ordered list of document ids that should live in `stack_id` after a
/// drag. Each id is upserted into that stack at its new position, so this single
/// command covers reordering within a column, moving a card between columns, and
/// dropping a brand-new document in from the library. Documents not present in
/// `document_ids` are left untouched (the column they moved to repositions them
/// with its own call), so a card never falls out of the project by accident.
/// Existing pile members are expanded to their complete pile before rewriting.
#[tauri::command]
pub(crate) fn set_project_document_order(
    app: AppHandle,
    project_id: String,
    stack_id: String,
    document_ids: Vec<String>,
) -> Result<Vec<ProjectDocument>, String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start reordering the project documents: {error}"))?;
    require_project_stack(&transaction, &project_id, &stack_id)?;
    let now = database::unix_timestamp();
    // A pile is always a board unit. If a document was dragged from the library
    // while it already belongs to a pile, expand it to the complete pile here so
    // a second representation of that paper cannot split the stored group.
    let mut normalized_document_ids = Vec::new();
    let mut seen_document_ids = HashSet::new();
    for document_id in document_ids {
        let pile_id: Option<String> = transaction
            .query_row(
                r#"
                SELECT pile_id
                FROM project_documents
                WHERE project_id = ?1 AND document_id = ?2
                "#,
                params![project_id, document_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("Could not inspect the project document pile: {error}"))?
            .flatten();
        let entry_document_ids = if let Some(pile_id) = pile_id {
            project_pile_document_ids(&transaction, &project_id, &pile_id)?
        } else {
            vec![document_id]
        };
        for entry_document_id in entry_document_ids {
            if seen_document_ids.insert(entry_document_id.clone()) {
                normalized_document_ids.push(entry_document_id);
            }
        }
    }
    for (position, document_id) in normalized_document_ids.iter().enumerate() {
        require_document(&transaction, document_id)?;
        transaction
            .execute(
                r#"
                INSERT INTO project_documents (
                    project_id, document_id, stack_id, position, added_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                ON CONFLICT(project_id, document_id) DO UPDATE SET
                    stack_id = excluded.stack_id,
                    position = excluded.position,
                    updated_at = excluded.updated_at
                "#,
                params![project_id, document_id, stack_id, position as i64, now],
            )
            .map_err(|error| format!("Could not reorder the project document: {error}"))?;
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not finish reordering the project documents: {error}"))?;
    emit_library_changed(&app, "projectDocument", None, "reordered");
    load_project_documents(&app, &connection, &project_id)
}

/// Combines one board entry with another. A source entry is either a singleton
/// document or every member of an existing pile; a library document can also be
/// supplied before it has project membership. The target stays the first visible
/// document and the source members are appended in their existing order.
#[tauri::command]
pub(crate) fn pile_project_documents(
    app: AppHandle,
    project_id: String,
    source_document_ids: Vec<String>,
    target_document_id: String,
) -> Result<Vec<ProjectDocument>, String> {
    if source_document_ids.is_empty() {
        return Err("A pile source is required".to_owned());
    }

    let mut seen = HashSet::new();
    let source_document_ids = source_document_ids
        .into_iter()
        .filter(|document_id| seen.insert(document_id.clone()))
        .collect::<Vec<_>>();
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start piling the project documents: {error}"))?;

    require_project(&transaction, &project_id)?;
    let (target_stack_id, target_pile_id): (String, Option<String>) = transaction
        .query_row(
            r#"
            SELECT stack_id, pile_id
            FROM project_documents
            WHERE project_id = ?1 AND document_id = ?2
            "#,
            params![project_id, target_document_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("Could not inspect the target paper: {error}"))?
        .ok_or_else(|| "The target paper is not in this project".to_owned())?;

    let target_members = if let Some(pile_id) = &target_pile_id {
        project_pile_document_ids(&transaction, &project_id, pile_id)?
    } else {
        vec![target_document_id.clone()]
    };
    if source_document_ids
        .iter()
        .any(|document_id| target_members.contains(document_id))
    {
        return load_project_documents(&app, &transaction, &project_id);
    }

    for document_id in &source_document_ids {
        require_document(&transaction, document_id)?;
    }

    let source_membership: Option<(String, Option<String>)> = transaction
        .query_row(
            r#"
            SELECT stack_id, pile_id
            FROM project_documents
            WHERE project_id = ?1 AND document_id = ?2
            "#,
            params![project_id, source_document_ids[0]],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("Could not inspect the source paper: {error}"))?;

    let (source_members, source_stack_id) = match source_membership {
        Some((stack_id, Some(pile_id))) => {
            let members = project_pile_document_ids(&transaction, &project_id, &pile_id)?;
            let member_set = members.iter().collect::<HashSet<_>>();
            if source_document_ids.len() != members.len()
                || source_document_ids
                    .iter()
                    .any(|document_id| !member_set.contains(document_id))
            {
                return Err("A paper pile must be moved as one unit".to_owned());
            }
            (members, Some(stack_id))
        }
        Some((stack_id, None)) => {
            if source_document_ids.len() != 1 {
                return Err("The selected papers do not form one pile".to_owned());
            }
            (source_document_ids, Some(stack_id))
        }
        None => {
            if source_document_ids.len() != 1 {
                return Err("Only one library paper can be added at a time".to_owned());
            }
            (source_document_ids, None)
        }
    };

    let target_column_before =
        project_stack_document_ids(&transaction, &project_id, &target_stack_id)?;
    let target_member_set = target_members.iter().cloned().collect::<HashSet<_>>();
    let source_member_set = source_members.iter().cloned().collect::<HashSet<_>>();
    let mut target_column_after = Vec::with_capacity(
        target_column_before.len()
            + source_members
                .iter()
                .filter(|document_id| !target_column_before.contains(document_id))
                .count(),
    );
    let mut inserted_pile = false;
    for document_id in target_column_before {
        if target_member_set.contains(&document_id) {
            if !inserted_pile {
                target_column_after.extend(target_members.iter().cloned());
                target_column_after.extend(source_members.iter().cloned());
                inserted_pile = true;
            }
        } else if !source_member_set.contains(&document_id) {
            target_column_after.push(document_id);
        }
    }
    if !inserted_pile {
        return Err("The target paper could not be placed".to_owned());
    }

    let pile_id = target_pile_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = database::unix_timestamp();
    for document_id in &target_members {
        transaction
            .execute(
                r#"
                UPDATE project_documents
                SET pile_id = ?1, updated_at = ?2
                WHERE project_id = ?3 AND document_id = ?4
                "#,
                params![pile_id, now, project_id, document_id],
            )
            .map_err(|error| format!("Could not update the target pile: {error}"))?;
    }
    for document_id in &source_members {
        transaction
            .execute(
                r#"
                INSERT INTO project_documents (
                    project_id, document_id, stack_id, pile_id, position, added_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5)
                ON CONFLICT(project_id, document_id) DO UPDATE SET
                    stack_id = excluded.stack_id,
                    pile_id = excluded.pile_id,
                    updated_at = excluded.updated_at
                "#,
                params![project_id, document_id, target_stack_id, pile_id, now],
            )
            .map_err(|error| format!("Could not add a paper to the pile: {error}"))?;
    }
    rewrite_project_stack_positions(
        &transaction,
        &project_id,
        &target_stack_id,
        &target_column_after,
        now,
    )?;

    if let Some(source_stack_id) = source_stack_id {
        if source_stack_id != target_stack_id {
            let source_column_after =
                project_stack_document_ids(&transaction, &project_id, &source_stack_id)?;
            rewrite_project_stack_positions(
                &transaction,
                &project_id,
                &source_stack_id,
                &source_column_after,
                now,
            )?;
        }
    }

    transaction
        .commit()
        .map_err(|error| format!("Could not finish piling the project documents: {error}"))?;
    emit_library_changed(&app, "projectDocument", None, "piled");
    load_project_documents(&app, &connection, &project_id)
}

#[tauri::command]
pub(crate) fn unpile_project_documents(
    app: AppHandle,
    project_id: String,
    pile_id: String,
) -> Result<Vec<ProjectDocument>, String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start unstacking the papers: {error}"))?;
    let changed = transaction
        .execute(
            r#"
            UPDATE project_documents
            SET pile_id = NULL, updated_at = ?1
            WHERE project_id = ?2 AND pile_id = ?3
            "#,
            params![database::unix_timestamp(), project_id, pile_id],
        )
        .map_err(|error| format!("Could not unstack the papers: {error}"))?;
    if changed == 0 {
        return Err("Paper pile not found".to_owned());
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not finish unstacking the papers: {error}"))?;
    emit_library_changed(&app, "projectDocument", None, "unpiled");
    load_project_documents(&app, &connection, &project_id)
}

#[tauri::command]
pub(crate) fn remove_document_from_project(
    app: AppHandle,
    project_id: String,
    document_id: String,
) -> Result<(), String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start removing the project document: {error}"))?;
    let changed = transaction
        .execute(
            "DELETE FROM project_documents WHERE project_id = ?1 AND document_id = ?2",
            params![project_id, document_id],
        )
        .map_err(|error| format!("Could not remove the document from the project: {error}"))?;
    if changed == 0 {
        return Err("Document is not in this project".to_owned());
    }
    clear_singleton_piles(&transaction, &project_id)?;
    transaction
        .commit()
        .map_err(|error| format!("Could not finish removing the project document: {error}"))?;
    emit_library_changed(&app, "projectDocument", Some(&document_id), "removed");
    Ok(())
}

#[tauri::command]
pub(crate) fn link_document_reference(
    app: AppHandle,
    document_id: String,
    reference_id: String,
) -> Result<LibraryDocument, String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start linking the document: {error}"))?;
    require_document(&transaction, &document_id)?;
    let reference_id = database::resolve_root_id(&transaction, &reference_id)?;
    let reference_exists = transaction
        .query_row(
            "SELECT 1 FROM \"references\" WHERE id = ?1 AND merged_into IS NULL",
            params![reference_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| format!("Could not validate the reference: {error}"))?
        .is_some();
    if !reference_exists {
        return Err("Reference not found".to_owned());
    }
    let linked_document: Option<String> = transaction
        .query_row(
            "SELECT document_id FROM document_reference_links WHERE reference_id = ?1",
            params![reference_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not check the reference link: {error}"))?;
    if linked_document
        .as_deref()
        .is_some_and(|id| id != document_id)
    {
        return Err("That reference already has a local PDF".to_owned());
    }
    transaction
        .execute(
            r#"
            INSERT INTO document_reference_links (document_id, reference_id, linked_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(document_id) DO UPDATE SET
                reference_id = excluded.reference_id,
                linked_at = excluded.linked_at
            "#,
            params![document_id, reference_id, database::unix_timestamp()],
        )
        .map_err(|error| format!("Could not link the document and reference: {error}"))?;
    transaction
        .execute(
            "UPDATE documents SET updated_at = ?1 WHERE id = ?2",
            params![database::unix_timestamp(), document_id],
        )
        .map_err(|error| format!("Could not update the document timestamp: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not finish linking the document: {error}"))?;
    let document = load_document(&app, &connection, &document_id)?;
    emit_library_changed(&app, "document", Some(&document_id), "updated");
    Ok(document)
}

#[tauri::command]
pub(crate) fn unlink_document_reference(
    app: AppHandle,
    document_id: String,
) -> Result<LibraryDocument, String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start unlinking the document: {error}"))?;
    require_document(&transaction, &document_id)?;
    transaction
        .execute(
            "DELETE FROM document_reference_links WHERE document_id = ?1",
            params![document_id],
        )
        .map_err(|error| format!("Could not unlink the document and reference: {error}"))?;
    transaction
        .execute(
            "UPDATE documents SET updated_at = ?1 WHERE id = ?2",
            params![database::unix_timestamp(), document_id],
        )
        .map_err(|error| format!("Could not update the document timestamp: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not finish unlinking the document: {error}"))?;
    let document = load_document(&app, &connection, &document_id)?;
    emit_library_changed(&app, "document", Some(&document_id), "updated");
    Ok(document)
}

#[tauri::command]
pub(crate) fn list_document_annotations(
    app: AppHandle,
    document_id: String,
) -> Result<Vec<DocumentAnnotation>, String> {
    let connection = database::connection(&app)?;
    require_document(&connection, &document_id)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, document_id, kind, page_index, color, opacity,
                   selected_text, annotation_json, created_at, updated_at
            FROM document_annotations
            WHERE document_id = ?1
            ORDER BY page_index, created_at, id
            "#,
        )
        .map_err(|error| format!("Could not prepare document annotations: {error}"))?;
    let annotations = statement
        .query_map(params![document_id], row_to_document_annotation)
        .map_err(|error| format!("Could not load document annotations: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read document annotations: {error}"))?;
    Ok(annotations)
}

#[tauri::command]
pub(crate) fn save_document_annotation(
    app: AppHandle,
    document_id: String,
    annotation: Value,
    selected_text: Option<String>,
) -> Result<DocumentAnnotation, String> {
    let annotation_id = string_field(&annotation, "id")?;
    let annotation_type = integer_field(&annotation, "type")?;
    if annotation_type != HIGHLIGHT_ANNOTATION_SUBTYPE {
        return Err("Only highlight annotations can be saved right now".to_owned());
    }
    let page_index = integer_field(&annotation, "pageIndex")?;
    if page_index < 0 {
        return Err("Annotation page index cannot be negative".to_owned());
    }
    let segment_rects = annotation
        .get("segmentRects")
        .and_then(Value::as_array)
        .ok_or_else(|| "Highlight annotation is missing segment rectangles".to_owned())?;
    if segment_rects.is_empty() {
        return Err("Highlight annotation must include at least one segment rectangle".to_owned());
    }
    let color = annotation
        .get("strokeColor")
        .or_else(|| annotation.get("color"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("#FFCD45")
        .to_owned();
    let opacity = annotation
        .get("opacity")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    let annotation_json = serde_json::to_string(&annotation)
        .map_err(|error| format!("Could not serialize the annotation: {error}"))?;

    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start saving the annotation: {error}"))?;
    require_document(&transaction, &document_id)?;

    if let Some(existing_document_id) = transaction
        .query_row(
            "SELECT document_id FROM document_annotations WHERE id = ?1",
            params![annotation_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| format!("Could not check the annotation id: {error}"))?
    {
        if existing_document_id != document_id {
            return Err("Annotation id already belongs to another document".to_owned());
        }
    }

    let now = database::unix_timestamp();
    let created_at = transaction
        .query_row(
            "SELECT created_at FROM document_annotations WHERE id = ?1 AND document_id = ?2",
            params![annotation_id, document_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| format!("Could not read the existing annotation: {error}"))?
        .unwrap_or(now);
    transaction
        .execute(
            r#"
            INSERT INTO document_annotations (
                id, document_id, kind, page_index, color, opacity,
                selected_text, annotation_json, created_at, updated_at
            ) VALUES (?1, ?2, 'highlight', ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                page_index = excluded.page_index,
                color = excluded.color,
                opacity = excluded.opacity,
                selected_text = excluded.selected_text,
                annotation_json = excluded.annotation_json,
                updated_at = excluded.updated_at
            "#,
            params![
                annotation_id,
                document_id,
                page_index,
                color,
                opacity,
                selected_text,
                annotation_json,
                created_at,
                now
            ],
        )
        .map_err(|error| format!("Could not save the annotation: {error}"))?;
    transaction
        .execute(
            "UPDATE documents SET updated_at = ?1 WHERE id = ?2",
            params![now, document_id],
        )
        .map_err(|error| format!("Could not update the document timestamp: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not finish saving the annotation: {error}"))?;
    load_document_annotation(&connection, &document_id, &annotation_id)
}

#[tauri::command]
pub(crate) fn delete_document_annotation(
    app: AppHandle,
    document_id: String,
    annotation_id: String,
) -> Result<(), String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start deleting the annotation: {error}"))?;
    require_document(&transaction, &document_id)?;
    let changed = transaction
        .execute(
            "DELETE FROM document_annotations WHERE document_id = ?1 AND id = ?2",
            params![document_id, annotation_id],
        )
        .map_err(|error| format!("Could not delete the annotation: {error}"))?;
    if changed == 0 {
        return Err("Annotation not found".to_owned());
    }
    transaction
        .execute(
            "UPDATE documents SET updated_at = ?1 WHERE id = ?2",
            params![database::unix_timestamp(), document_id],
        )
        .map_err(|error| format!("Could not update the document timestamp: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not finish deleting the annotation: {error}"))?;
    Ok(())
}

fn load_document(
    app: &AppHandle,
    connection: &Connection,
    id: &str,
) -> Result<LibraryDocument, String> {
    let row = connection
        .query_row(
            r#"
            SELECT d.content_hash, d.original_filename, d.title, d.byte_size,
                   l.reference_id, r.data_json, d.created_at, d.updated_at, d.last_viewed_at
            FROM documents d
            LEFT JOIN document_reference_links l ON l.document_id = d.id
            LEFT JOIN "references" r ON r.id = l.reference_id
            WHERE d.id = ?1
            "#,
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .optional()
        .map_err(|error| format!("Could not load the document: {error}"))?
        .ok_or_else(|| "Document not found".to_owned())?;
    let linked_reference = row
        .5
        .as_deref()
        .map(serde_json::from_str::<LinkedReferenceData>)
        .transpose()
        .map_err(|error| format!("Could not read linked reference metadata: {error}"))?;
    let thumbnail_path = crate::thumbnail::thumbnail_path(app, &row.0)
        .ok()
        .filter(|path| path.exists())
        .map(|path| path.to_string_lossy().into_owned());
    Ok(LibraryDocument {
        id: id.to_owned(),
        content_hash: row.0,
        original_filename: row.1,
        title: row.2,
        byte_size: row.3.max(0) as u64,
        stored_path: document_path(app, id)?.to_string_lossy().into_owned(),
        thumbnail_path,
        reference_id: row.4,
        reference_title: linked_reference
            .as_ref()
            .and_then(|reference| reference.title.clone()),
        reference_authors: linked_reference
            .map(|reference| reference.authors)
            .unwrap_or_default(),
        created_at: row.6,
        updated_at: row.7,
        last_viewed_at: row.8,
    })
}

fn load_project(connection: &Connection, id: &str) -> Result<Project, String> {
    connection
        .query_row(
            r#"
            SELECT id, name, created_at, updated_at, last_opened_at,
                   (SELECT COUNT(*) FROM project_documents pd WHERE pd.project_id = projects.id)
            FROM projects WHERE id = ?1
            "#,
            params![id],
            row_to_project,
        )
        .optional()
        .map_err(|error| format!("Could not load the project: {error}"))?
        .ok_or_else(|| "Project not found".to_owned())
}

fn load_project_stack(
    connection: &Connection,
    project_id: &str,
    stack_id: &str,
) -> Result<ProjectStack, String> {
    connection
        .query_row(
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM project_stacks
            WHERE project_id = ?1 AND id = ?2
            "#,
            params![project_id, stack_id],
            row_to_project_stack,
        )
        .optional()
        .map_err(|error| format!("Could not load the project stack: {error}"))?
        .ok_or_else(|| "Project stack not found".to_owned())
}

fn load_project_document(
    app: &AppHandle,
    connection: &Connection,
    project_id: &str,
    document_id: &str,
) -> Result<ProjectDocument, String> {
    let row = connection
        .query_row(
            r#"
            SELECT ps.id, ps.project_id, ps.name, ps.created_at, ps.updated_at,
                   pd.pile_id, pd.position, pd.added_at, pd.updated_at
            FROM project_documents pd
            JOIN project_stacks ps ON ps.project_id = pd.project_id AND ps.id = pd.stack_id
            WHERE pd.project_id = ?1 AND pd.document_id = ?2
            "#,
            params![project_id, document_id],
            |row| {
                Ok((
                    ProjectStack {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        name: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    },
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .optional()
        .map_err(|error| format!("Could not load the project document: {error}"))?
        .ok_or_else(|| "Document is not in this project".to_owned())?;
    Ok(ProjectDocument {
        project_id: project_id.to_owned(),
        document: load_document(app, connection, document_id)?,
        stack: row.0,
        pile_id: row.1,
        position: row.2,
        added_at: row.3,
        updated_at: row.4,
    })
}

fn project_stack_document_ids(
    connection: &Connection,
    project_id: &str,
    stack_id: &str,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            r#"
            SELECT document_id
            FROM project_documents
            WHERE project_id = ?1 AND stack_id = ?2
            ORDER BY position, document_id
            "#,
        )
        .map_err(|error| format!("Could not prepare the project stack order: {error}"))?;
    let document_ids = statement
        .query_map(params![project_id, stack_id], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not read the project stack order: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not collect the project stack order: {error}"))?;
    Ok(document_ids)
}

fn project_pile_document_ids(
    connection: &Connection,
    project_id: &str,
    pile_id: &str,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            r#"
            SELECT document_id
            FROM project_documents
            WHERE project_id = ?1 AND pile_id = ?2
            ORDER BY position, document_id
            "#,
        )
        .map_err(|error| format!("Could not prepare the paper pile: {error}"))?;
    let document_ids = statement
        .query_map(params![project_id, pile_id], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not read the paper pile: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not collect the paper pile: {error}"))?;
    Ok(document_ids)
}

fn rewrite_project_stack_positions(
    connection: &Connection,
    project_id: &str,
    stack_id: &str,
    document_ids: &[String],
    now: i64,
) -> Result<(), String> {
    for (position, document_id) in document_ids.iter().enumerate() {
        connection
            .execute(
                r#"
                UPDATE project_documents
                SET position = ?1, updated_at = ?2
                WHERE project_id = ?3 AND stack_id = ?4 AND document_id = ?5
                "#,
                params![position as i64, now, project_id, stack_id, document_id],
            )
            .map_err(|error| format!("Could not rewrite the project stack order: {error}"))?;
    }
    Ok(())
}

fn clear_singleton_piles(connection: &Connection, project_id: &str) -> Result<(), String> {
    connection
        .execute(
            r#"
            UPDATE project_documents
            SET pile_id = NULL
            WHERE project_id = ?1
              AND pile_id IS NOT NULL
              AND pile_id IN (
                  SELECT pile_id
                  FROM project_documents
                  WHERE project_id = ?1 AND pile_id IS NOT NULL
                  GROUP BY pile_id
                  HAVING COUNT(*) < 2
              )
            "#,
            params![project_id],
        )
        .map_err(|error| format!("Could not clean up empty paper piles: {error}"))?;
    Ok(())
}

fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        last_opened_at: row.get(4)?,
        document_count: row.get(5)?,
    })
}

fn row_to_project_stack(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectStack> {
    Ok(ProjectStack {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn load_document_annotation(
    connection: &Connection,
    document_id: &str,
    annotation_id: &str,
) -> Result<DocumentAnnotation, String> {
    connection
        .query_row(
            r#"
            SELECT id, document_id, kind, page_index, color, opacity,
                   selected_text, annotation_json, created_at, updated_at
            FROM document_annotations
            WHERE document_id = ?1 AND id = ?2
            "#,
            params![document_id, annotation_id],
            row_to_document_annotation,
        )
        .optional()
        .map_err(|error| format!("Could not load the saved annotation: {error}"))?
        .ok_or_else(|| "Annotation not found".to_owned())
}

fn row_to_document_annotation(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocumentAnnotation> {
    let annotation_json: String = row.get(7)?;
    let annotation = serde_json::from_str(&annotation_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let page_index = row.get::<_, i64>(3)?.max(0) as u32;
    Ok(DocumentAnnotation {
        id: row.get(0)?,
        document_id: row.get(1)?,
        kind: row.get(2)?,
        page_index,
        color: row.get(4)?,
        opacity: row.get(5)?,
        selected_text: row.get(6)?,
        annotation,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub(crate) fn all_document_ids(app: &AppHandle) -> Result<Vec<String>, String> {
    let connection = database::connection(app)?;
    let mut statement = connection
        .prepare("SELECT id FROM documents")
        .map_err(|error| format!("Could not prepare the document id list: {error}"))?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not list document ids: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read document ids: {error}"))?;
    Ok(ids)
}

// (document_id, content_hash) pairs for the thumbnail backfill sweep.
pub(crate) fn all_document_hashes(app: &AppHandle) -> Result<Vec<(String, String)>, String> {
    let connection = database::connection(app)?;
    let mut statement = connection
        .prepare("SELECT id, content_hash FROM documents")
        .map_err(|error| format!("Could not prepare the document hash list: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("Could not list document hashes: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read document hashes: {error}"))?;
    Ok(rows)
}

fn document_id_by_hash(connection: &Connection, hash: &str) -> Result<Option<String>, String> {
    connection
        .query_row(
            "SELECT id FROM documents WHERE content_hash = ?1",
            params![hash],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not check whether the PDF is already imported: {error}"))
}

fn require_document(connection: &Connection, id: &str) -> Result<(), String> {
    let exists = connection
        .query_row("SELECT 1 FROM documents WHERE id = ?1", params![id], |_| {
            Ok(())
        })
        .optional()
        .map_err(|error| format!("Could not validate the document: {error}"))?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err("Document not found".to_owned())
    }
}

fn require_project(connection: &Connection, id: &str) -> Result<(), String> {
    let exists = connection
        .query_row("SELECT 1 FROM projects WHERE id = ?1", params![id], |_| {
            Ok(())
        })
        .optional()
        .map_err(|error| format!("Could not validate the project: {error}"))?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err("Project not found".to_owned())
    }
}

fn require_project_stack(
    connection: &Connection,
    project_id: &str,
    stack_id: &str,
) -> Result<(), String> {
    let exists = connection
        .query_row(
            "SELECT 1 FROM project_stacks WHERE project_id = ?1 AND id = ?2",
            params![project_id, stack_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| format!("Could not validate the project stack: {error}"))?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err("Project stack not found".to_owned())
    }
}

fn string_field(value: &Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("Annotation is missing {field}"))
}

fn integer_field(value: &Value, field: &str) -> Result<i64, String> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("Annotation is missing numeric {field}"))
}

pub(crate) fn document_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    let directory = database::app_data_directory(app)?.join("documents");
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create the document library directory: {error}"))?;
    Ok(directory.join(format!("{id}.pdf")))
}

fn looks_like_pdf(bytes: &[u8]) -> bool {
    bytes
        .get(..bytes.len().min(1024))
        .is_some_and(|prefix| prefix.windows(5).any(|window| window == b"%PDF-"))
}

fn clean_name(value: &str) -> Option<String> {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!value.is_empty()).then_some(value)
}

fn name_key(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_pdf_headers_within_the_first_kilobyte() {
        assert!(looks_like_pdf(b"%PDF-1.7\n"));
        assert!(looks_like_pdf(b"prefix\n%PDF-1.4\n"));
        assert!(!looks_like_pdf(b"not a pdf"));
    }

    #[test]
    fn normalizes_stack_names() {
        assert_eq!(
            clean_name("  Machine   Learning ").as_deref(),
            Some("Machine Learning")
        );
        assert_eq!(name_key("Machine Learning"), "machine learning");
    }
}
