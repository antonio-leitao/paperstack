use super::{database, reference_resolver};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use tauri::AppHandle;
use uuid::Uuid;

const HIGHLIGHT_ANNOTATION_SUBTYPE: i64 = 9;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Stack {
    id: String,
    name: String,
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
    reference_id: Option<String>,
    stacks: Vec<Stack>,
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
    load_document(&app, &connection, &id)
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
    load_document(&app, &connection, &id)
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
    load_document(&app, &connection, &id)
}

#[tauri::command]
pub(crate) fn delete_document(app: AppHandle, id: String) -> Result<(), String> {
    let mut connection = database::connection(&app)?;
    let stored_path = document_path(&app, &id)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start deleting the document: {error}"))?;
    let changed = transaction
        .execute("DELETE FROM documents WHERE id = ?1", params![id])
        .map_err(|error| format!("Could not delete the document: {error}"))?;
    if changed == 0 {
        return Err("Document not found".to_owned());
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not finish deleting the document: {error}"))?;
    match std::fs::remove_file(stored_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "The document record was deleted, but its PDF could not be removed: {error}"
        )),
    }
}

#[tauri::command]
pub(crate) fn create_stack(app: AppHandle, name: String) -> Result<Stack, String> {
    let name = clean_name(&name).ok_or_else(|| "Stack name cannot be empty".to_owned())?;
    let id = Uuid::new_v4().to_string();
    let connection = database::connection(&app)?;
    connection
        .execute(
            "INSERT INTO stacks (id, name, name_key, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, name_key(&name), database::unix_timestamp()],
        )
        .map_err(|error| {
            format!("Could not create the stack; its name may already exist: {error}")
        })?;
    Ok(Stack { id, name })
}

#[tauri::command]
pub(crate) fn list_stacks(app: AppHandle) -> Result<Vec<Stack>, String> {
    let connection = database::connection(&app)?;
    let mut statement = connection
        .prepare("SELECT id, name FROM stacks ORDER BY name_key, id")
        .map_err(|error| format!("Could not prepare the stack list: {error}"))?;
    let stacks = statement
        .query_map([], |row| {
            Ok(Stack {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|error| format!("Could not list stacks: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read the stack list: {error}"))?;
    Ok(stacks)
}

#[tauri::command]
pub(crate) fn rename_stack(app: AppHandle, id: String, name: String) -> Result<Stack, String> {
    let name = clean_name(&name).ok_or_else(|| "Stack name cannot be empty".to_owned())?;
    let connection = database::connection(&app)?;
    let changed = connection
        .execute(
            "UPDATE stacks SET name = ?1, name_key = ?2 WHERE id = ?3",
            params![name, name_key(&name), id],
        )
        .map_err(|error| {
            format!("Could not rename the stack; its name may already exist: {error}")
        })?;
    if changed == 0 {
        return Err("Stack not found".to_owned());
    }
    Ok(Stack { id, name })
}

#[tauri::command]
pub(crate) fn delete_stack(app: AppHandle, id: String) -> Result<(), String> {
    let connection = database::connection(&app)?;
    let changed = connection
        .execute("DELETE FROM stacks WHERE id = ?1", params![id])
        .map_err(|error| format!("Could not delete the stack: {error}"))?;
    if changed == 0 {
        return Err("Stack not found".to_owned());
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn set_document_stacks(
    app: AppHandle,
    document_id: String,
    stack_ids: Vec<String>,
) -> Result<LibraryDocument, String> {
    let mut connection = database::connection(&app)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start updating document stacks: {error}"))?;
    require_document(&transaction, &document_id)?;
    let stack_ids = stack_ids.into_iter().collect::<BTreeSet<_>>();
    for stack_id in &stack_ids {
        let exists = transaction
            .query_row(
                "SELECT 1 FROM stacks WHERE id = ?1",
                params![stack_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| format!("Could not validate a stack: {error}"))?
            .is_some();
        if !exists {
            return Err(format!("Stack not found: {stack_id}"));
        }
    }
    transaction
        .execute(
            "DELETE FROM document_stacks WHERE document_id = ?1",
            params![document_id],
        )
        .map_err(|error| format!("Could not clear document stacks: {error}"))?;
    for stack_id in stack_ids {
        transaction
            .execute(
                "INSERT INTO document_stacks (document_id, stack_id) VALUES (?1, ?2)",
                params![document_id, stack_id],
            )
            .map_err(|error| format!("Could not attach a stack: {error}"))?;
    }
    transaction
        .execute(
            "UPDATE documents SET updated_at = ?1 WHERE id = ?2",
            params![database::unix_timestamp(), document_id],
        )
        .map_err(|error| format!("Could not update the document timestamp: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not finish updating document stacks: {error}"))?;
    load_document(&app, &connection, &document_id)
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
    load_document(&app, &connection, &document_id)
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
    load_document(&app, &connection, &document_id)
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
    let mut statement = connection
        .prepare(
            r#"
            SELECT s.id, s.name
            FROM stacks s
            JOIN document_stacks ds ON ds.stack_id = s.id
            WHERE ds.document_id = ?1
            ORDER BY s.name_key, s.id
            "#,
        )
        .map_err(|error| format!("Could not prepare document stacks: {error}"))?;
    let stacks = statement
        .query_map(params![id], |row| {
            Ok(Stack {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|error| format!("Could not load document stacks: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read document stacks: {error}"))?;
    let linked_reference = row
        .5
        .as_deref()
        .map(serde_json::from_str::<LinkedReferenceData>)
        .transpose()
        .map_err(|error| format!("Could not read linked reference metadata: {error}"))?;
    Ok(LibraryDocument {
        id: id.to_owned(),
        content_hash: row.0,
        original_filename: row.1,
        title: row.2,
        byte_size: row.3.max(0) as u64,
        stored_path: document_path(app, id)?.to_string_lossy().into_owned(),
        reference_id: row.4,
        stacks,
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
        rusqlite::Error::FromSqlConversionFailure(
            7,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
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

fn document_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
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
