///! Handles the CLI definition and parsing.
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[clap(
    name = "graphctl",
    version = "v0.1.0",
    author = "Austin Poor",
    about = "A CLI for interacting with a local graph database",
    long_about = ""
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,

    #[clap(
        long,
        global = true,
        env = "GRAPHCTL_CONFIG_DIR",
        help = "Path to the config directory. Defaults to $HOME/.graphctl"
    )]
    pub config_dir: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Create a node or edge in the graph")]
    Create {
        #[command(subcommand)]
        cmd: CreateCmd,
    },

    #[clap(about = "List nodes or edges from the graph")]
    List {
        #[command(subcommand)]
        cmd: ListCmd,
    },

    #[clap(about = "Get a node or edge from the graph")]
    Get {
        #[command(subcommand)]
        cmd: GetCmd,
    },

    #[clap(about = "Update a node or edge in the graph")]
    Update {
        #[command(subcommand)]
        cmd: UpdateCmd,
    },

    #[clap(about = "Delete a node or edge from the graph")]
    Delete {
        #[command(subcommand)]
        cmd: DeleteCmd,
    },

    /// This may be able to do stuff like create-/view-schemas, etc.
    #[clap(about = "Meta graph commands")]
    Meta,

    #[clap(about = "Configure the graphctl CLI")]
    Cfg {
        #[clap(subcommand)]
        cmd: CfgCmd,
    },
}

#[derive(Subcommand, Debug)]
pub enum CreateCmd {
    #[clap(about = "Create a node in the graph")]
    Node(CreateNodeArgs),

    #[clap(about = "Create an edge in the graph")]
    Edge(CreateEdgeArgs),
}

#[derive(Args, Debug)]
pub struct CreateNodeArgs {
    #[clap(short, long, num_args=0.., help = "The node's label")]
    pub label: Vec<String>,

    #[clap(short, long, num_args=0.., help="A property attached to the node")]
    pub prop: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CreateEdgeArgs {
    #[clap(short, long, help = "The edge's type")]
    pub edge_type: String,

    #[clap(short, long, help = "The edge's source node")]
    pub from_node: String,

    #[clap(short, long, help = "The edge's target node")]
    pub to_node: String,

    #[clap(short, long, help = "Whether the edge is directed.")]
    pub directed: bool,

    #[clap(short, long, num_args=0.., help="A property on the edge")]
    pub prop: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum ListCmd {
    #[clap(about = "List nodes in the graph")]
    Nodes(ListNodesArgs),

    #[clap(about = "List edges in the graph")]
    Edges(ListEdgesArgs),
}

#[derive(Args, Debug)]
pub struct ListNodesArgs {
    #[clap(long, help = "The node's label")]
    pub has_label: Option<String>,

    #[clap(long, num_args=0.., help = "Filter to nodes with a certain property")]
    pub has_prop: Vec<String>,

    #[clap(short, long, num_args=0.., help = "Filter to nodes with a key-value pair")]
    pub prop: Vec<String>,

    #[clap(short='o', long, num_args=0.., help = "IDs of edges going out")]
    pub edge_out: Vec<String>,

    #[clap(short='i', long, num_args=0.., help = "IDs of edges coming in")]
    pub edge_in: Vec<String>,

    #[clap(long, num_args=0.., help = "Key-value pairs of edges out. Either `EDGE_TYPE=:NodeLabel` or `EDGE_TYPE=node-id`")]
    pub edge_out_to: Vec<String>,

    #[clap(long, num_args=0.., help = "Key-value pairs of edges in. Either `EDGE_TYPE=:NodeLabel` or `EDGE_TYPE=node-id`")]
    pub edge_in_from: Vec<String>,

    #[clap(short, long, help = "Count the number of nodes returned")]
    pub count: bool,

    #[clap(short, long, help = "Limit the number of nodes returned")]
    pub limit: Option<usize>,

    #[clap(short, long, help = "Output format", value_enum, default_value_t=OutputFormat::Json)]
    pub format: OutputFormat,
}

#[derive(Args, Debug)]
pub struct ListEdgesArgs {
    #[clap(long, help = "The edge's type")]
    pub has_label: Option<String>,

    #[clap(long, num_args=0.., help = "Filter to edges with a certain property")]
    pub has_prop: Vec<String>,

    #[clap(short, long, num_args=0.., help = "Filter to edges with property key-value match")]
    pub prop: Vec<String>,

    #[clap(short, long, help = "ID of the source node")]
    pub source_node: Option<String>,

    #[clap(short, long, help = "ID of the target node")]
    pub target_node: Option<String>,

    #[clap(short, long, help = "Count the number of edges returned")]
    pub count: bool,

    #[clap(short, long, help = "Limit the number of nodes returned")]
    pub limit: Option<usize>,

    #[clap(short, long, help = "Output format", value_enum, default_value_t=OutputFormat::Json)]
    pub format: OutputFormat,
}

#[derive(Subcommand, Debug)]
pub enum GetCmd {
    #[clap(about = "Get a node from the graph")]
    Node(GetNodeArgs),

    #[clap(about = "Get a edge from the graph")]
    Edge(GetEdgeArgs),
}

#[derive(Args, Debug)]
pub struct GetNodeArgs {
    #[clap(short, long, help = "The node's ID")]
    pub id: String,

    #[clap(short, long, help = "Show the node's properties")]
    pub props: bool,

    #[clap(short, long, help = "Show the node's incoming edges")]
    pub edges_in: bool,

    #[clap(short, long, help = "Show the node's outgoing edges")]
    pub edges_out: bool,
}

#[derive(Args, Debug)]
pub struct GetEdgeArgs {
    #[clap(short, long, help = "The edge's ID")]
    pub id: String,

    #[clap(short, long, help = "Show the edge's properties")]
    pub props: bool,
}

#[derive(Subcommand, Debug)]
pub enum UpdateCmd {
    #[clap(about = "Update nodes in the graph")]
    Node(UpdateNodeArgs),

    #[clap(about = "Update edges in the graph")]
    Edge(UpdateEdgeArgs),
}

#[derive(Args, Debug)]
pub struct UpdateNodeArgs {
    #[clap(short, long, help = "The node's ID")]
    pub id: String,

    #[clap(short, long, help = "Labels to add to the node")]
    pub add_label: Vec<String>,

    #[clap(short, long, help = "Labels to remove from the node")]
    pub remove_label: Vec<String>,

    #[clap(short, long, help = "Props to set on the node")]
    pub set_prop: Vec<String>,

    #[clap(short, long, help = "Props to remove from the node")]
    pub remove_prop: Vec<String>,
}

#[derive(Args, Debug)]
pub struct UpdateEdgeArgs {
    #[clap(short, long, help = "The edge's ID")]
    pub id: String,

    #[clap(short, long, help = "Set the edge's type")]
    pub edge_type: Option<String>,

    #[clap(short, long, help = "Set the edge's source node")]
    pub from_node: Option<String>,

    #[clap(short, long, help = "Set the edge's target node")]
    pub to_node: Option<String>,

    #[clap(short, long, help = "Set the edge as directed")]
    pub set_directed: bool,

    #[clap(short, long, help = "Set the edge as undirected")]
    pub set_undirected: bool,

    #[clap(short, long, help = "Props to set on the edge")]
    pub set_prop: Vec<String>,

    #[clap(short, long, help = "Props to remove from the edge")]
    pub remove_prop: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum DeleteCmd {
    #[clap(about = "Delete nodes from the graph")]
    Node(DeleteNodeArgs),

    #[clap(about = "Delete edges from the graph")]
    Edge(DeleteEdgeArgs),
}

#[derive(Args, Debug)]
pub struct DeleteNodeArgs {
    #[clap(short, long, help = "The node's ID")]
    pub id: String,
}

#[derive(Args, Debug)]
pub struct DeleteEdgeArgs {
    #[clap(short, long, help = "The edge's ID")]
    pub id: String,
}

#[derive(Subcommand, Debug)]
pub enum CfgCmd {
    #[clap(about = "Initialize the graphctl CLI")]
    Init,

    #[clap(about = "Get the database type")]
    GetDbType(GetDbTypeArgs),

    #[clap(about = "Set the database type")]
    SetDbType(SetDbTypeArgs),

    #[clap(about = "Get the remote database URL")]
    GetRemoteDbUrl(GetDbTypeArgs),

    #[clap(about = "Set the remote database URL")]
    SetRemoteDbUrl(SetRemoteDbUrlArgs),

    #[clap(about = "Get the remote database auth token")]
    GetRemoteDbToken(GetRemoteDbTokenArgs),

    #[clap(about = "Set the remote database auth token")]
    SetRemoteDbToken(SetRemoteDbTokenArgs),

    #[clap(about = "Get the local database encryption key")]
    GetEncryptionKey(GetEncryptionKeyArgs),

    #[clap(about = "Set the local database encryption key")]
    SetEncryptionKey(SetEncryptionKeyArgs),
}

#[derive(Args, Debug)]
pub struct GetDbTypeArgs;

#[derive(Args, Debug)]
pub struct SetDbTypeArgs;

#[derive(Args, Debug)]
pub struct GetRemoteDbUrlArgs;

#[derive(Args, Debug)]
pub struct SetRemoteDbUrlArgs {
    #[clap(short, long, help = "The URL of the remote database")]
    pub url: String,
}

#[derive(Args, Debug)]
pub struct GetRemoteDbTokenArgs;

#[derive(Args, Debug)]
pub struct SetRemoteDbTokenArgs {
    #[clap(short, long, help = "The auth token for the remote database")]
    pub token: String,
}

#[derive(Args, Debug)]
pub struct GetEncryptionKeyArgs;

#[derive(Args, Debug)]
pub struct SetEncryptionKeyArgs {
    #[clap(short, long, help = "The encryption key for the local database")]
    pub key: String,
}

#[derive(Debug, Default, Clone, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Json,
    Ndjson,
    Table,
}
