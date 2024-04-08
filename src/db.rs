#![allow(dead_code, unused_variables)]
///! Handles the connection to the database.
use super::conf::{Config, DBType, DB_DIR_NAME, DB_FILE_NAME};
use super::secrets::{get_local_db_encryption_key, get_remote_db_auth_token};
use crate::util;
use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use chrono::{DateTime, Local};
use libsql::{de, Builder, Cipher, Connection, Database, EncryptionConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Using the given configuration, connect to the database.
pub async fn connect_to_db(conf_path: &PathBuf, config: &Config) -> Result<Database> {
    let db = match config.db.db_type {
        DBType::Local => connect_to_local_db(conf_path, config.db.encrypt_replica).await?,
        DBType::RemoteOnly => {
            let url = config
                .db
                .remote_db_path
                .as_ref()
                .ok_or_else(|| anyhow!("No remote database path set."))?;
            connect_to_remote_db(url).await?
        }
        DBType::RemoteWithReplica => {
            let url = config
                .db
                .remote_db_path
                .as_ref()
                .ok_or_else(|| anyhow!("No remote database path set."))?;
            connect_to_remote_with_replica_db(conf_path, url, config.db.encrypt_replica).await?
        }
    };
    Ok(db)
}

async fn connect_to_local_db(conf_path: &PathBuf, encrypt: bool) -> Result<Database> {
    // Get the local path...
    let local_path = conf_path.join(DB_DIR_NAME).join(DB_FILE_NAME);

    // Create the builder...
    let mut builder = Builder::new_local(local_path);

    // Should it be encrypted?
    if encrypt {
        // Get the encryption key (as bytes)...
        let keys = get_local_db_encryption_key()?;
        let keyb = Bytes::from(keys);

        // Add it to the builder...
        builder = builder.encryption_config(EncryptionConfig {
            cipher: Cipher::Aes256Cbc,
            encryption_key: keyb,
        });
    }

    // Build and return...
    Ok(builder.build().await?)
}

async fn connect_to_remote_db(remote_path: &str) -> Result<Database> {
    // Get the remote auth token...
    let auth_token = get_remote_db_auth_token()?;

    // Create the builder...
    let builder = Builder::new_remote(remote_path.to_string(), auth_token);

    // Build and return...
    Ok(builder.build().await?)
}

async fn connect_to_remote_with_replica_db(
    conf_path: &PathBuf,
    remote_path: &str,
    encrypt: bool,
) -> Result<Database> {
    // Get the local path...
    let local_path = conf_path.join(DB_DIR_NAME).join(DB_FILE_NAME);

    // Get the auth token...
    let auth_token = get_remote_db_auth_token()?;

    // Create the builder...
    let mut builder = Builder::new_remote_replica(local_path, remote_path.to_string(), auth_token);

    // Should it be encrypted?
    if encrypt {
        // Get the encryption key (as bytes)...
        let keys = get_local_db_encryption_key()?;
        let keyb = Bytes::from(keys);

        // Add it to the builder...
        builder = builder.encryption_config(EncryptionConfig {
            cipher: Cipher::Aes256Cbc,
            encryption_key: keyb,
        });
    }

    // Build and return...
    Ok(builder.build().await?)
}

/// Initialize the database.
pub async fn init_db(conn: &Connection) -> Result<()> {
    // Get the migration count...
    let count = get_migration_count(conn).await?;

    // Run the migrations...
    if count < 1 {
        migrations_v1(conn).await?;
        set_migration_count(conn, 1).await?;
    }

    // Note - Future migrations will go here...
    // ...

    // Done!
    Ok(())
}

/// Gets the migration count from the database.
async fn get_migration_count(conn: &Connection) -> Result<i64> {
    // Create the meta table if it doesn't already exist...
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _meta (
            key     TEXT PRIMARY KEY, 
            val_txt TEXT,
            val_int INTEGER
        );",
        (),
    )
    .await
    .with_context(|| format!("Failed to create meta table"))?; // TODO - Add context...

    // Get the migration count...
    let mut rows = conn
        .prepare("SELECT val_int FROM _meta WHERE key = 'migration_count';")
        .await?
        .query(())
        .await?;

    // There should be either zero or one rows...
    if let Some(row) = rows.next().await? {
        let val = row.get_value(0)?;
        if let libsql::Value::Integer(v) = val {
            return Ok(v);
        }
        return Err(anyhow!("Invalid migration count value"));
    }

    // Otherwise, insert the value...
    conn.execute(
        "INSERT INTO _meta (key, val_int) VALUES ('migration_count', 0);",
        (),
    )
    .await?;

    // And return it...
    Ok(0)
}

/// Set the migration count in the database.
async fn set_migration_count(conn: &Connection, count: u32) -> Result<()> {
    // TODO - Add error context...
    conn.execute(
        "
        UPDATE _meta 
        SET val_int = ? 
        WHERE key = 'migration_count';
        ",
        [count],
    )
    .await?;
    Ok(())
}

pub async fn migrations_v1(conn: &Connection) -> Result<()> {
    // Create the node table...
    // TODO - Add error context...
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS nodes (
            id         TEXT PRIMARY KEY, 
            labels     TEXT NOT NULL,
            created_at TEXT NOT NULL, 
            updated_at TEXT NOT NULL
        );",
        (),
    )
    .await?;

    // Create the node table...
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS node_props (
            node_id    TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            key        TEXT NOT NULL,
            value      TEXT NOT NULL,
            created_at TEXT NOT NULL, 
            updated_at TEXT NOT NULL,
            PRIMARY KEY (node_id, key)
        );",
        (),
    )
    .await?;

    // Create the node table...
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS edges (
            id         TEXT PRIMARY KEY, 
            edge_type  TEXT NOT NULL,
            from_node  TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            to_node    TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            directed   INT  NOT NULL,
            created_at TEXT NOT NULL, 
            updated_at TEXT NOT NULL
        );",
        (),
    )
    .await?;

    // Create the node table...
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS edge_props (
            edge_id    TEXT NOT NULL REFERENCES edges(id) ON DELETE CASCADE,
            key        TEXT NOT NULL,
            value      TEXT NOT NULL,
            created_at TEXT NOT NULL, 
            updated_at TEXT NOT NULL,
            PRIMARY KEY (edge_id, key)
        );",
        (),
    )
    .await?;

    // Done!
    Ok(())
}

/// The database representation of a node.
#[derive(Debug, Serialize, Deserialize)]
pub struct DbNode {
    pub id: String,
    pub labels: Vec<String>,
    pub props: Option<HashMap<String, Value>>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// The database representation of an edge.
#[derive(Debug, Serialize, Deserialize)]
pub struct DbEdge {
    pub id: String,
    pub edge_type: String,
    pub from_node: String,
    pub to_node: String,
    pub directed: bool,
    pub props: Option<HashMap<String, Value>>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

pub struct CreateNodeParams {
    pub labels: Vec<String>,
    pub props: HashMap<String, Value>,
}

pub async fn create_node(conn: &Connection, params: &CreateNodeParams) -> Result<DbNode> {
    // Generate an ID and timestamp...
    let id = util::new_id("n");
    let now = Local::now();

    // Convert the node type and timestamp to a SQL value...
    let labels = serde_json::to_string(&params.labels)?;
    let sql_now = libsql::Value::Text(now.to_rfc3339());

    // Start a transaction...
    let tx = conn.transaction().await?;

    // Insert the node...
    tx.execute(
        "
        INSERT INTO nodes (
            id, 
            labels, 
            created_at, 
            updated_at
        ) VALUES (?, ?, ?, ?);
        ",
        libsql::params![id.clone(), labels, sql_now.clone(), sql_now.clone(),],
    )
    .await?;

    // Add the properties...
    for (key, value) in params.props.iter() {
        let sql_key = libsql::Value::Text(key.trim().to_string());
        let sql_value = libsql::Value::Text(value.to_string());
        tx.execute(
            "
            INSERT INTO node_props (
                node_id, 
                key, 
                value, 
                created_at, 
                updated_at
            ) VALUES (?, ?, ?, ?, ?);
            ",
            libsql::params![
                id.clone(),
                sql_key,
                sql_value,
                sql_now.clone(),
                sql_now.clone(),
            ],
        )
        .await?;
    }

    // Commit the transaction...
    tx.commit().await?;

    // Return the data...
    Ok(DbNode {
        id,
        labels: params.labels.clone(),
        created_at: now,
        updated_at: now,
        props: Some(params.props.clone()),
    })
}

pub struct CreateEdgeParams {
    pub edge_type: String,
    pub from_node: String,
    pub to_node: String,
    pub directed: bool,
    pub props: HashMap<String, Value>,
}

pub async fn create_edge(conn: &Connection, params: &CreateEdgeParams) -> Result<DbEdge> {
    // Generate an ID and timestamp...
    let id = util::new_id("e");
    let now = Local::now();

    // Convert the timestamp to a SQL value...
    let sql_now = libsql::Value::Text(now.to_rfc3339());

    // Start a transaction...
    let tx = conn.transaction().await?;

    // Insert the edge...
    tx.execute(
        "
        INSERT INTO edges (
            id, 
            edge_type, 
            from_node, 
            to_node, 
            directed, 
            created_at, 
            updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?);
        ",
        libsql::params![
            id.clone(),
            params.edge_type.clone(),
            params.from_node.clone(),
            params.to_node.clone(),
            params.directed as i64,
            sql_now.clone(),
            sql_now.clone(),
        ],
    )
    .await?;

    // Add the properties...
    for (key, value) in params.props.iter() {
        let sql_key = libsql::Value::Text(key.trim().to_lowercase());
        let sql_value = libsql::Value::Text(value.to_string());
        tx.execute(
            "
            INSERT INTO edge_props (
                edge_id, 
                key, 
                value, 
                created_at, 
                updated_at
            ) VALUES (?, ?, ?, ?, ?);
            ",
            libsql::params![
                id.clone(),
                sql_key,
                sql_value,
                sql_now.clone(),
                sql_now.clone(),
            ],
        )
        .await?;
    }

    // Commit the transaction...
    tx.commit().await?;

    // Return the data...
    Ok(DbEdge {
        id,
        edge_type: params.edge_type.clone(),
        from_node: params.from_node.clone(),
        to_node: params.to_node.clone(),
        directed: params.directed,
        created_at: now,
        updated_at: now,
        props: Some(params.props.clone()),
    })
}

pub struct ListNodesParams;

pub async fn list_nodes(conn: &Connection, params: &ListNodesParams) -> Result<Vec<DbNode>> {
    let mut res = conn
        .prepare(
            "
            SELECT id, labels, created_at, updated_at
            FROM nodes;
            "
        )
        .await?
        .query(libsql::params![])
        .await?;
    
    let mut nodes = Vec::new();
    while let Some(row) = res.next().await? {
        // let node = de::from_row::<DbNode>(&row)?;

        // Get the values...
        let id: String = row.get(0)?;
        let slabels: String = row.get(1)?;
        let labels: Vec<String> = serde_json::from_str(&slabels)?;
        let created_at: DateTime<Local> = row.get::<String>(2)?.parse()?;
        let updated_at: DateTime<Local> = row.get::<String>(3)?.parse()?;
       
        // Get the props...
        let props = get_node_props(conn, &id).await?;

        // Add it to the list...
        nodes.push(DbNode {
            id,
            labels,
            props: Some(props),
            created_at,
            updated_at,
        });
    }

    Ok(nodes)
}

pub struct ListEdgesParams;

pub async fn list_edges(conn: &Connection, params: &ListEdgesParams) -> Result<Vec<DbEdge>> {
    let mut res = conn
        .prepare(
            "
            SELECT id, edge_type, from_node, to_node, directed, created_at, updated_at
            FROM edges;
            "
        )
        .await?
        .query(libsql::params![])
        .await?;
    
    let mut edges = Vec::new();
    while let Some(row) = res.next().await? {
        // Get the values...
        let mut e = de::from_row::<DbEdge>(&row)?;

        // Get the props...
        let props = get_edge_props(conn, &e.id).await?;
        e.props = Some(props);

        // Add it to the list...
        edges.push(e);
    }

    Ok(edges)
}

pub async fn check_node_exists(conn: &Connection, id: &str) -> Result<bool> {
    let res = conn
        .prepare(
            "
            SELECT COUNT(*) > 0
            FROM nodes 
            WHERE id = ?;
            ",
        )
        .await?
        .query_row(libsql::params![id])
        .await?;
    Ok(res.get(0)?)
}

pub async fn check_edge_exists(conn: &Connection, id: &str) -> Result<bool> {
    let res = conn
        .prepare(
            "
            SELECT COUNT(*) > 0
            FROM edges
            WHERE id = ?;
            ",
        )
        .await?
        .query_row(libsql::params![id])
        .await?;
    Ok(res.get(0)?)
}

pub struct GetNodeParams {
    pub id: String,
    pub with_props: bool,
    // pub with_edges: bool,
}

pub async fn get_node(conn: &Connection, params: &GetNodeParams) -> Result<DbNode> {
    // Get the node...
    let row = conn
        .prepare(
            "
            SELECT id, node_type, created_at, updated_at 
            FROM nodes 
            WHERE id = ?;
            ",
        )
        .await?
        .query_row(libsql::params![params.id.clone()])
        .await?;

    // Get the values...
    let mut node = de::from_row::<DbNode>(&row)?;

    // Get the properties?
    if params.with_props {
        let props = get_node_props(conn, &params.id).await?;
        node.props = Some(props);
    }

    // Return the data!
    Ok(node)
}

pub async fn get_node_props(conn: &Connection, node_id: &str) -> Result<HashMap<String, Value>> {
    // Query the props in the database...
    let mut rows = conn
        .prepare(
            "
            SELECT key, value 
            FROM node_props 
            WHERE node_id = ?;
            ",
        )
        .await?
        .query(libsql::params![node_id])
        .await?;

    // Add them to a map...
    let mut map = HashMap::new();
    while let Some(row) = rows.next().await? {
        let key: String = row.get(0)?;
        let value: String = row.get(1)?;
        map.insert(key, serde_json::from_str(&value)?);
    }

    // Return the data!
    Ok(map)
}

pub struct GetEdgeParams {
    pub id: String,
    pub with_props: bool,
}

pub async fn get_edge(conn: &Connection, params: &GetEdgeParams) -> Result<DbEdge> {
    // Get the edge...
    let row = conn
        .prepare(
            "
            SELECT id, edge_type, from_node, to_node, directed, created_at, updated_at
            FROM edges
            WHERE id = ?;
            ",
        )
        .await?
        .query_row(libsql::params![params.id.clone()])
        .await?;

    // Get the values...
    let mut edge = de::from_row::<DbEdge>(&row)?;

    // Get the properties?
    if params.with_props {
        let props = get_edge_props(conn, &params.id).await?;
        edge.props = Some(props);
    }

    // Return the data!
    Ok(edge)
}

pub async fn get_edge_props(conn: &Connection, edge_id: &str) -> Result<HashMap<String, Value>> {
    // Query the props in the database...
    let mut rows = conn
        .prepare(
            "
            SELECT key, value 
            FROM edge_props 
            WHERE edge_id = ?;
            ",
        )
        .await?
        .query(libsql::params![edge_id])
        .await?;

    // Add them to a map...
    let mut map = HashMap::new();
    while let Some(row) = rows.next().await? {
        let key: String = row.get(0)?;
        let value: String = row.get(1)?;
        map.insert(key, serde_json::from_str(&value)?);
    }

    // Return the data!
    Ok(map)
}

pub async fn get_node_edges_in(conn: &Connection, node_id: &str) -> Result<Vec<String>> {
    // Query the props in the database...
    let mut rows = conn
        .prepare(
            "
            SELECT id 
            FROM edges
            WHERE to_node = ? OR (NOT directed AND from_node = ?);
            ",
        )
        .await?
        .query(libsql::params![node_id, node_id,])
        .await?;

    // Add them to a map...
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let key: String = row.get(0)?;
        out.push(key);
    }

    // Return the data!
    Ok(out)
}

pub async fn get_node_edges_out(conn: &Connection, node_id: &str) -> Result<Vec<String>> {
    // Query the props in the database...
    let mut rows = conn
        .prepare(
            "
            SELECT id 
            FROM edges
            WHERE from_node = ? OR (NOT directed AND to_node = ?);
            ",
        )
        .await?
        .query(libsql::params![node_id, node_id,])
        .await?;

    // Add them to a map...
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let key: String = row.get(0)?;
        out.push(key);
    }

    // Return the data!
    Ok(out)
}

pub async fn update_node(conn: &Connection) -> Result<DbNode> {
    todo!();
}

pub async fn set_node_prop(conn: &Connection) -> Result<()> {
    todo!();
}

pub async fn update_edge(conn: &Connection) -> Result<DbEdge> {
    todo!();
}

pub async fn set_edge_prop(conn: &Connection) -> Result<()> {
    todo!();
}

pub async fn delete_node(conn: &Connection) -> Result<()> {
    todo!();
}

pub async fn delete_node_prop(conn: &Connection) -> Result<()> {
    todo!();
}

pub async fn delete_edge(conn: &Connection) -> Result<()> {
    todo!();
}

pub async fn delete_edge_prop(conn: &Connection) -> Result<()> {
    todo!();
}
