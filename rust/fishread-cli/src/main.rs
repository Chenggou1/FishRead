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
}

#[derive(Debug, Subcommand)]
enum ChapterCommand {
    /// List chapters of the current book
    List,
}

#[derive(Debug, Subcommand)]
enum ReadCommand {
    /// Read the current chunk
    Current,
    /// Advance to the next chunk
    Next,
    /// Go back to the previous chunk
    Prev,
}

fn main() {
    let cli = Cli::parse();

    let (json, exit_code) = match cli.command {
        Command::Init => commands::init::run(),
        Command::Import { path } => commands::import::run(&path),
        Command::Book { sub } => match sub {
            BookCommand::List => commands::book::list(),
            BookCommand::Use { book_id } => commands::book::use_book(&book_id),
        },
        Command::Chapter { sub } => match sub {
            ChapterCommand::List => commands::chapter::list(),
        },
        Command::Read { sub } => match sub {
            ReadCommand::Current => commands::read::current(),
            ReadCommand::Next => commands::read::next(),
            ReadCommand::Prev => commands::read::prev(),
        },
    };

    println!("{json}");
    std::process::exit(exit_code);
}

