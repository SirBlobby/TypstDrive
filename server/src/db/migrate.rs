use std::collections::HashSet;

use sqlx::{AnyConnection, AnyPool, AssertSqlSafe, Connection};

use super::dialect::Dialect;
use super::schema::{tables, Column, CopyRule, Table, BACKFILLS, INDEXES, LEGACY_RENAMES, RETIRED_TABLES};

pub const CURRENT_VERSION: i64 = 13;
const REBUILD_SUFFIX: &str = "__rebuild";

async fn run(connection: &mut AnyConnection, sql: String) -> Result<(), sqlx::Error> {
    sqlx::query(AssertSqlSafe(sql)).execute(&mut *connection).await.map(|_| ())
}

async fn table_exists(connection: &mut AnyConnection, dialect: Dialect, table: &str) -> Result<bool, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(dialect.table_exists_query())
        .bind(table)
        .fetch_one(&mut *connection)
        .await?;
    Ok(count > 0)
}

async fn column_names(connection: &mut AnyConnection, dialect: Dialect, table: &str) -> Result<HashSet<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(dialect.column_names_query())
        .bind(table)
        .fetch_all(&mut *connection)
        .await?;
    Ok(rows.into_iter().map(|(name,)| name).collect())
}

async fn stored_version(connection: &mut AnyConnection) -> Result<Option<i64>, sqlx::Error> {
    run(connection, "CREATE TABLE IF NOT EXISTS schema_version (version BIGINT NOT NULL)".to_string()).await?;
    let (version,): (Option<i64>,) = sqlx::query_as("SELECT MAX(version) FROM schema_version")
        .fetch_one(&mut *connection)
        .await?;
    Ok(version)
}

fn mentions_column(column_list: &str, column: &str) -> bool {
    let Some(open) = column_list.find('(') else {
        return false;
    };
    let close = column_list[open..].find(')').map(|offset| open + offset).unwrap_or(column_list.len());
    column_list[open + 1..close].split(',').any(|name| name.trim() == column)
}

fn is_key_column(table: &Table, column: &str) -> bool {
    let in_constraint = table.constraints.iter().any(|constraint| mentions_column(constraint, column));
    let in_index = INDEXES
        .iter()
        .filter(|index| index.contains(&format!(" ON {} ", table.name)))
        .any(|index| mentions_column(&index[index.find(" ON ").unwrap_or(0)..], column));
    in_constraint || in_index
}

fn split_reference(dialect: Dialect, definition: String) -> (String, Option<String>) {
    if dialect != Dialect::Mysql {
        return (definition, None);
    }
    match definition.find(" REFERENCES ") {
        Some(position) => (definition[..position].to_string(), Some(definition[position + 1..].to_string())),
        None => (definition, None),
    }
}

fn column_sql(dialect: Dialect, table: &Table, column: &Column, created_name: &str) -> (String, Option<String>) {
    let rendered = dialect.render_column(column.definition, created_name, is_key_column(table, column.name));
    let (definition, reference) = split_reference(dialect, rendered);
    let foreign_key = reference.map(|reference| format!("FOREIGN KEY ({}) {reference}", column.name));
    (format!("{} {definition}", column.name), foreign_key)
}

fn create_table_sql(dialect: Dialect, table: &Table, created_name: &str) -> String {
    let mut parts = Vec::new();
    let mut foreign_keys = Vec::new();
    for column in table.columns {
        let (definition, foreign_key) = column_sql(dialect, table, column, created_name);
        parts.push(definition);
        foreign_keys.extend(foreign_key);
    }
    parts.extend(table.constraints.iter().map(|constraint| constraint.to_string()));
    parts.extend(foreign_keys);
    format!(
        "CREATE TABLE IF NOT EXISTS {created_name} (\n    {}\n){}",
        parts.join(",\n    "),
        dialect.table_options()
    )
}

async fn add_missing_columns(connection: &mut AnyConnection, dialect: Dialect) -> Result<(), sqlx::Error> {
    for table in tables() {
        let existing_columns = column_names(connection, dialect, table.name).await?;
        for column in table.columns.iter().filter(|column| !existing_columns.contains(column.name)) {
            let (definition, foreign_key) = column_sql(dialect, table, column, table.name);
            let mut statement = format!("ALTER TABLE {} ADD COLUMN {definition}", table.name);
            if let Some(foreign_key) = foreign_key {
                statement.push_str(&format!(", ADD {foreign_key}"));
            }
            run(connection, statement).await?;
        }
    }
    Ok(())
}

async fn run_backfills(connection: &mut AnyConnection, upgraded_from: i64) -> Result<(), sqlx::Error> {
    for backfill in BACKFILLS.iter().filter(|backfill| backfill.since_version > upgraded_from) {
        run(connection, backfill.statement.to_string()).await?;
    }
    Ok(())
}

fn copy_expression(dialect: Dialect, column: &Column) -> String {
    let name = column.name;
    match column.copy {
        CopyRule::Plain | CopyRule::RequiredReference(_) => name.to_string(),
        CopyRule::DefaultWhenNull(fallback) => format!("COALESCE({name}, {})", dialect.render(fallback, "")),
        CopyRule::Boolean => format!("CASE WHEN {name} THEN 1 ELSE 0 END"),
        CopyRule::OptionalReference(parent) => {
            format!("CASE WHEN {name} IN (SELECT id FROM {parent}) THEN {name} ELSE NULL END")
        }
    }
}

fn copy_rows_sql(dialect: Dialect, table: &Table, existing_columns: &HashSet<String>) -> Option<String> {
    let copied: Vec<&Column> = table
        .columns
        .iter()
        .filter(|column| existing_columns.contains(column.name))
        .collect();
    if copied.is_empty() {
        return None;
    }

    let target_columns: Vec<&str> = copied.iter().map(|column| column.name).collect();
    let source_expressions: Vec<String> = copied.iter().map(|column| copy_expression(dialect, column)).collect();
    let required_filters: Vec<String> = copied
        .iter()
        .filter_map(|column| match column.copy {
            CopyRule::RequiredReference(parent) => Some(format!("{} IN (SELECT id FROM {parent})", column.name)),
            _ => None,
        })
        .collect();

    let where_clause = if required_filters.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", required_filters.join(" AND "))
    };

    Some(format!(
        "INSERT INTO {}{REBUILD_SUFFIX} ({}) SELECT {} FROM {}{where_clause}",
        table.name,
        target_columns.join(", "),
        source_expressions.join(", "),
        table.name
    ))
}

async fn create_fresh_schema(connection: &mut AnyConnection, dialect: Dialect) -> Result<(), sqlx::Error> {
    for table in tables() {
        run(connection, create_table_sql(dialect, table, table.name)).await?;
    }
    Ok(())
}

async fn rebuild_table(connection: &mut AnyConnection, dialect: Dialect, table: &Table) -> Result<(), sqlx::Error> {
    if !table_exists(connection, dialect, table.name).await? {
        return run(connection, create_table_sql(dialect, table, table.name)).await;
    }

    let rebuild_name = format!("{}{REBUILD_SUFFIX}", table.name);
    run(connection, dialect.drop_table(&rebuild_name)).await?;
    run(connection, create_table_sql(dialect, table, &rebuild_name)).await?;

    let existing_columns = column_names(connection, dialect, table.name).await?;
    if let Some(copy_sql) = copy_rows_sql(dialect, table, &existing_columns) {
        run(connection, copy_sql).await?;
    }

    run(connection, dialect.drop_table(table.name)).await?;
    run(connection, format!("ALTER TABLE {rebuild_name} RENAME TO {}", table.name)).await
}

async fn upgrade_legacy_schema(connection: &mut AnyConnection, dialect: Dialect) -> Result<(), sqlx::Error> {
    for statement in LEGACY_RENAMES {
        let mut savepoint = connection.begin().await?;
        if sqlx::query(*statement).execute(&mut *savepoint).await.is_ok() {
            savepoint.commit().await?;
        } else {
            savepoint.rollback().await?;
        }
    }

    for table in tables() {
        rebuild_table(connection, dialect, table).await?;
    }

    for table in RETIRED_TABLES {
        run(connection, dialect.drop_table(table)).await?;
    }
    Ok(())
}

async fn apply_migrations(connection: &mut AnyConnection, dialect: Dialect) -> Result<(), sqlx::Error> {
    let version = stored_version(connection).await?;
    if version.is_some_and(|version| version >= CURRENT_VERSION) {
        return Ok(());
    }

    let has_existing_data = table_exists(connection, dialect, "users").await?;
    let mut transaction = connection.begin().await?;

    if let Some(previous_version) = version {
        tracing::info!("Adding new tables and columns for schema version {CURRENT_VERSION}");
        create_fresh_schema(&mut transaction, dialect).await?;
        add_missing_columns(&mut transaction, dialect).await?;
        run_backfills(&mut transaction, previous_version).await?;
    } else if has_existing_data {
        tracing::info!("Upgrading existing database to schema version {CURRENT_VERSION}");
        upgrade_legacy_schema(&mut transaction, dialect).await?;
        run_backfills(&mut transaction, 0).await?;
    } else {
        tracing::info!("Creating database schema version {CURRENT_VERSION}");
        create_fresh_schema(&mut transaction, dialect).await?;
    }

    run(&mut transaction, "DELETE FROM schema_version".to_string()).await?;
    sqlx::query(super::sql("INSERT INTO schema_version (version) VALUES ($1)"))
        .bind(CURRENT_VERSION)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await
}

async fn create_index(connection: &mut AnyConnection, dialect: Dialect, statement: &str) -> Result<(), sqlx::Error> {
    let Some(exists_query) = dialect.index_exists_query() else {
        return run(connection, statement.to_string()).await;
    };
    let index_name = statement.split_whitespace().nth(5).unwrap_or_default();
    let (count,): (i64,) = sqlx::query_as(exists_query)
        .bind(index_name)
        .fetch_one(&mut *connection)
        .await?;
    if count > 0 {
        return Ok(());
    }
    run(connection, statement.replace(" IF NOT EXISTS", "")).await
}

pub async fn migrate(pool: &AnyPool, dialect: Dialect) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;

    match dialect {
        Dialect::Sqlite => {
            run(&mut connection, "PRAGMA journal_mode = WAL".to_string()).await?;
            run(&mut connection, "PRAGMA foreign_keys = OFF".to_string()).await?;
        }
        Dialect::Mysql => run(&mut connection, "SET FOREIGN_KEY_CHECKS = 0".to_string()).await?,
        Dialect::Postgres => {}
    }

    let result = apply_migrations(&mut connection, dialect).await;

    match dialect {
        Dialect::Sqlite => run(&mut connection, "PRAGMA foreign_keys = ON".to_string()).await?,
        Dialect::Mysql => run(&mut connection, "SET FOREIGN_KEY_CHECKS = 1".to_string()).await?,
        Dialect::Postgres => {}
    }
    result?;

    for index in INDEXES {
        create_index(&mut connection, dialect, index).await?;
    }

    Ok(())
}
