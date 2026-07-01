mod commands;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "fishread", about = "Local EPUB reading runtime", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize the FishRead database
    Init,

    /// Import an EPUB file into the library
    Import {
        /// Path to the EPUB file
        path: String,
    },

    /// Manage books in the library
    Book {
        #[command(subcommand)]
        sub: BookCommand,
    },

    /// Manage chapters
    Chapter {
        #[command(subcommand)]
        sub: ChapterCommand,
    },

    /// Read content
    Read {
        #[command(subcommand)]
        sub: ReadCommand,
    },
}

#[derive(Debug, Subcommand)]
enum BookCommand {
    /// List all books in the library
    List,
    /// Set the current book
    Use {
        /// Book ID to select
        book_id: String,
    },
    /// Delete a book and its local reading state
    Delete {
        /// Book ID to delete
        book_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum ChapterCommand {
    /// List chapters of the current book
    List {
        /// Include reading anchors for navigation
        #[arg(long)]
        navigation: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ReadCommand {
    /// Read the current chunk
    Current,
    /// Advance to the next chunk
    Next,
    /// Go back to the previous chunk
    Prev,
    /// Jump to a specific reading position
    Jump {
        /// Chapter index to jump to
        #[arg(long)]
        chapter_index: i64,
        /// Chunk index to jump to
        #[arg(long)]
        chunk_index: i64,
    },
}

fn main() {
    let cli = Cli::parse();

    let (json, exit_code) = match cli.command {
        Command::Init => commands::init::run(),
        Command::Import { path } => commands::import::run(&path),
        Command::Book { sub } => match sub {
            BookCommand::List => commands::book::list(),
            BookCommand::Use { book_id } => commands::book::use_book(&book_id),
            BookCommand::Delete { book_id } => commands::book::delete_book(&book_id),
        },
        Command::Chapter { sub } => match sub {
            ChapterCommand::List { navigation } => commands::chapter::list(navigation),
        },
        Command::Read { sub } => match sub {
            ReadCommand::Current => commands::read::current(),
            ReadCommand::Next => commands::read::next(),
            ReadCommand::Prev => commands::read::prev(),
            ReadCommand::Jump {
                chapter_index,
                chunk_index,
            } => commands::read::jump(chapter_index, chunk_index),
        },
    };

    println!("{json}");
    std::process::exit(exit_code);
}
