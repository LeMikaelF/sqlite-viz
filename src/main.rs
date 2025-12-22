use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

use sqlite_viz::{Database, print_database_info};

#[derive(Parser)]
#[command(name = "sqlite-viz")]
#[command(author, version, about = "SQLite B-tree visualization tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate HTML visualization of SQLite database B-tree structure
    Viz {
        /// Path to SQLite database file
        #[arg(value_name = "DATABASE")]
        database: PathBuf,

        /// Output HTML file path (default: <database>.html)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Filter to specific table(s) - can be specified multiple times
        #[arg(short, long)]
        table: Option<Vec<String>>,

        /// Filter to specific index(es) - can be specified multiple times
        #[arg(short, long)]
        index: Option<Vec<String>>,
    },

    /// Display information about database structure
    Info {
        /// Path to SQLite database file
        #[arg(value_name = "DATABASE")]
        database: PathBuf,

        /// Show detailed information including schema and B-tree stats
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Viz { database, output, table, index } => {
            let db = Database::open(&database)?;

            let output_path = output.unwrap_or_else(|| {
                let mut path = database.clone();
                path.set_extension("html");
                path
            });

            let filter_tables = table.as_deref();
            let filter_indexes = index.as_deref();

            db.generate_visualization(&output_path, filter_tables, filter_indexes)?;

            println!("Visualization generated: {}", output_path.display());
        }

        Commands::Info { database, verbose } => {
            let db = Database::open(&database)?;
            print_database_info(&db, verbose);
        }
    }

    Ok(())
}
