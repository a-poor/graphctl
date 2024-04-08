mod cli;
mod conf;
mod db;
mod prompt;
mod secrets;
mod util;

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use cli::{CfgCmd, Cli, Commands, CreateCmd, DeleteCmd, GetCmd, ListCmd, UpdateCmd};
use conf::Config;
use db::{connect_to_db, init_db};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Load the CLI...
    let app = Cli::parse();

    // Load the config...
    let conf_dir = match conf::get_config_dir(app.config_dir) {
        Some(cd) => cd,
        None => {
            eprintln!("Error: Could not determine config directory.");
            std::process::exit(1);
        }
    };
    
    // Is this a init command?
    if matches!(app.cmd, Commands::Cfg { cmd: CfgCmd::Init }) {
        // Check that the config dir doesn't already exist...
        if conf_dir.exists() {
            eprintln!(
                "Error: Config directory \"{}\" already exists.",
                conf_dir.display(),
            );
            std::process::exit(1);
        }

        // Prompt for the database type...
        let db_type = prompt::prompt_for_db_type()?;

        // Get the remote path if needed...
        let remote_db_path = match db_type {
            conf::DBType::RemoteOnly | conf::DBType::RemoteWithReplica => {
                Some(prompt::prompt_for_remote_db_url()?)
            }
            _ => None,
        };

        // Get the encryption key if needed...
        match db_type {
            conf::DBType::RemoteOnly | conf::DBType::RemoteWithReplica => {
                // Promopt for the encryption key...
                let encryption_key = prompt::prompt_for_remote_db_auth_token()?;

                // Store the encryption key...
                secrets::set_remote_db_auth_token(&encryption_key)?;
            }
            _ => (),
        }

        // Should the local db be encrypted?
        let encrypt_local = match db_type {
            conf::DBType::Local => prompt::prompt_for_encrypt_local()?,
            conf::DBType::RemoteWithReplica => prompt::prompt_for_encrypt_replica()?,
            _ => false,
        };

        // If encrypting, generate a random key and store it...
        if encrypt_local {
            let key = secrets::generate_random_hex_string()?;
            secrets::set_local_db_encryption_key(&key)?;
        }

        // Store that data in the config...
        let cfg = Config {
            conf_dir,
            db: conf::DbConfig {
                db_type,
                remote_db_path,
                encrypt_replica: encrypt_local,
            },
        };

        // Create the config directory...
        if let Err(err) = std::fs::create_dir_all(&cfg.conf_dir) {
            eprintln!(
                "Error: Could not create config directory \"{}\": {}",
                cfg.conf_dir.display(),
                err,
            );
            std::process::exit(1);
        }

        // Write the config file...
        if let Err(err) = cfg.write_to_file() {
            eprintln!("Error: Could not write config file: {}", err,);
            std::process::exit(1);
        }

        // Make the data directory...
        let data_dir = cfg.conf_dir.join(conf::DB_DIR_NAME);
        if let Err(err) = std::fs::create_dir(&data_dir) {
            eprintln!(
                "Error: Could not create data directory \"{}\": {}",
                data_dir.display(),
                err,
            );
            std::process::exit(1);
        }

        // Create the db...
        let db = match connect_to_db(&cfg.conf_dir, &cfg).await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: Could not initialize database: {}", e);
                std::process::exit(1);
            }
        };

        // Create a connection...
        let conn = match db.connect() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Could not connect to database: {}", e);
                std::process::exit(1);
            }
        };

        // Run the migrations...
        if let Err(err) = init_db(&conn).await {
            eprintln!("Error: Could not initialize database: {}", err);
            std::process::exit(1);
        }

        // Done!
        return Ok(());
    }

    // Now make the config variable immutable...
    let cfg = Config::read_from_file(Some(conf_dir.to_string_lossy().to_string()))
        .context("Could not read config file.")?;

    // Make sure the config directory already exists...
    if !cfg.conf_dir.exists()  {
        eprintln!(
            "Error: Config directory \"{}\" doesn't exist. Run `graphctl init` to create it",
            cfg.conf_dir.display(),
        );
        std::process::exit(1);
    }
    
    // Make sure the config directory is a directory...
    if !cfg.conf_dir.is_dir() {
        eprintln!(
            "Error: Config directory \"{}\" exists but isn't a directory.
Remove it and then run `graphctl init` to create it",
            cfg.conf_dir.display(),
        );
        std::process::exit(1);
    }

    // Create the db...
    let db = match connect_to_db(&cfg.conf_dir, &cfg).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: Could not initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Create a connection...
    let conn = match db.connect() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Could not connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Run the migrations...
    if let Err(err) = init_db(&conn).await {
        eprintln!("Error: Could not initialize database: {}", err);
        std::process::exit(1);
    }

    // Handle the other commands...
    match app.cmd {
        Commands::Create { cmd } => match cmd {
            CreateCmd::Node(args) => {
                // TODO - Add output formatting options...

                // Split the props into key-value pairs...
                let mut props = HashMap::new();
                for p in args.prop {
                    // Split the key-value pair on on the equals sign...
                    let mut parts = p.splitn(2, '=');

                    // Get the key, strip, and convert to lowercase...
                    let key = parts
                        .next()
                        .ok_or(anyhow!("Failed to parse key-value pair."))
                        .context(format!("argument={}", p))?
                        .trim()
                        .to_string();

                    // Make sure the key is not empty...
                    if key.is_empty() {
                        return Err(anyhow!("Empty key in key-value pair."));
                    }

                    // Get the value...
                    let value = parts
                        .next()
                        .ok_or(anyhow!("Failed to parse key-value pair."))
                        .context(format!("argument={}", p))?;

                    // Try to parse it as JSON first,
                    // otherwise just use the string...
                    let value = match serde_json::from_str(value) {
                        Ok(v) => v,
                        Err(_) => serde_json::Value::String(value.to_string()),
                    };

                    // Add it to the props map...
                    props.insert(key, value);
                }

                // Add the node to the database...
                let res = db::create_node(
                    &conn,
                    &db::CreateNodeParams {
                        labels: args.label,
                        props,
                    },
                )
                .await?;

                // Print the result...
                println!("{}", serde_json::to_string_pretty(&res)?);
            }
            CreateCmd::Edge(args) => {
                // TODO - Add output formatting options...

                // Check that the source and target nodes exist...
                if !db::check_node_exists(&conn, &args.from_node).await? {
                    return Err(anyhow!("Source node does not exist."));
                }
                if args.from_node != args.to_node
                    && !db::check_node_exists(&conn, &args.to_node).await?
                {
                    return Err(anyhow!("Source node does not exist."));
                }

                // Split the props into key-value pairs...
                let mut props = HashMap::new();
                for p in args.prop.iter() {
                    // Split the key-value pair on on the equals sign...
                    let mut parts = p.splitn(2, '=');

                    // Get the key, strip, and convert to lowercase...
                    let key = parts
                        .next()
                        .ok_or(anyhow!("Failed to parse key-value pair."))
                        .context(format!("argument={}", p))?
                        .trim()
                        .to_string();

                    // Make sure the key is not empty...
                    if key.is_empty() {
                        return Err(anyhow!("Empty key in key-value pair."));
                    }

                    // Get the value...
                    let value = parts
                        .next()
                        .ok_or(anyhow!("Failed to parse key-value pair."))
                        .context(format!("argument={}", p))?;

                    // Try to parse it as JSON first,
                    // otherwise just use the string...
                    let value = match serde_json::from_str(value) {
                        Ok(v) => v,
                        Err(_) => serde_json::Value::String(value.to_string()),
                    };

                    // Add it to the props map...
                    props.insert(key, value);
                }

                // Create the edge...
                let res = db::create_edge(
                    &conn,
                    &db::CreateEdgeParams {
                        edge_type: args.edge_type,
                        from_node: args.from_node,
                        to_node: args.to_node,
                        directed: args.directed,
                        props,
                    },
                )
                .await?;

                // Print the result...
                println!("{}", serde_json::to_string_pretty(&res)?);
            }
        },
        Commands::List { cmd } => match cmd {
            ListCmd::Nodes(args) => {
                println!("Listing nodes. Args: {:?}", args);
            }
            ListCmd::Edges(args) => {
                println!("Listing edges. Args: {:?}", args);
            }
        },
        Commands::Get { cmd } => match cmd {
            GetCmd::Node(args) => {
                // Get the node...
                let res = db::get_node(&conn, &db::GetNodeParams {
                    id: args.id.clone(),
                    with_props: args.props,
                }).await?;

                // Get the node's edges in and out...
                let edges_in = match args.edges_in {
                    false => None,
                    true => Some(db::get_node_edges_in(&conn, &args.id.clone()).await?),
                };
                let edges_out = match args.edges_out {
                    false => None,
                    true => Some(db::get_node_edges_out(&conn, &args.id.clone()).await?),
                };

                // Print the result...
                let data = json!({
                    "id": res.id,
                    "labels": res.labels,
                    "props": res.props,
                    "edges_in": edges_in,
                    "edges_out": edges_out,
                    "created_at": res.created_at,
                    "updated_at": res.updated_at,
                });
                println!("{}", serde_json::to_string_pretty(&data)?);

            }
            GetCmd::Edge(args) => {
                // Get the edge...
                let res = db::get_edge(&conn, &db::GetEdgeParams{
                    id: args.id,
                    with_props: args.props,
                }).await?;

                // Print the result...
                println!("{}", serde_json::to_string_pretty(&res)?);
            }
        },
        Commands::Update { cmd } => match cmd {
            UpdateCmd::Node(args) => {
                println!("Updating a node. Args: {:?}", args);
            }
            UpdateCmd::Edge(args) => {
                println!("Updating an edge. Args: {:?}", args);
            }
        },
        Commands::Delete { cmd } => match cmd {
            DeleteCmd::Node(args) => {
                println!("Deleting a node. Args: {:?}", args);
            }
            DeleteCmd::Edge(args) => {
                println!("Deleting an edge. Args: {:?}", args);
            }
        },
        Commands::Meta => todo!("Meta command not yet implemented"),
        Commands::Cfg { cmd } => match cmd {
            CfgCmd::Init => unreachable!("Already handled init command"),
            CfgCmd::GetDbType(args) => {
                println!("Getting DB type. Args: {:?}", args);
            }
            CfgCmd::SetDbType(args) => {
                println!("Setting DB type. Args: {:?}", args);
            }
            CfgCmd::GetRemoteDbUrl(args) => {
                println!("Getting remote DB URL. Args: {:?}", args);
            }
            CfgCmd::SetRemoteDbUrl(args) => {
                println!("Setting remote DB URL. Args: {:?}", args);
            }
            CfgCmd::GetRemoteDbToken(args) => {
                println!("Getting remote DB auth token. Args: {:?}", args);
            }
            CfgCmd::SetRemoteDbToken(args) => {
                println!("Setting remote DB auth token. Args: {:?}", args);
            }
            CfgCmd::GetEncryptionKey(args) => {
                println!(
                    "Getting local db / local replica encryption key. Args: {:?}",
                    args
                );
            }
            CfgCmd::SetEncryptionKey(args) => {
                println!(
                    "Setting local db / local replica encryption key. Args: {:?}",
                    args
                );
            }
        },
    }

    // Done!
    Ok(())
}
