use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

use sqlite_viz::{Database, print_database_info, dump};

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

    /// Dump database structure to a human-readable text file for debugging
    Dump {
        /// Path to SQLite database file
        #[arg(value_name = "DATABASE")]
        database: PathBuf,

        /// Output text file path (default: <database>.dump.txt)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Dump specific B-tree(s) by name - can be specified multiple times
        #[arg(short = 't', long)]
        tree: Option<Vec<String>>,

        /// Dump specific page(s) by number - can be specified multiple times
        #[arg(short, long)]
        page: Option<Vec<u32>>,

        /// Omit hex dumps from output
        #[arg(long)]
        no_hex: bool,
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

        Commands::Dump { database, output, tree, page, no_hex } => {
            let db = Database::open(&database)?;

            let output_path = output.unwrap_or_else(|| {
                let mut path = database.clone();
                let new_name = format!(
                    "{}.dump.txt",
                    path.file_stem().and_then(|s| s.to_str()).unwrap_or("database")
                );
                path.set_file_name(new_name);
                path
            });

            let options = dump::DumpOptions {
                btrees: tree,
                pages: page,
                no_hex,
            };

            dump::dump_to_file(&db, &output_path, &options)?;

            println!("Dump written to: {}", output_path.display());
        }
    }

    Ok(())
}
