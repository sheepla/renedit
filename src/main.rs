use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(short, long, help = "Command of text editor")]
    #[clap(env = "EDITOR")]
    editor: String,

    #[arg(short, long, help = "Path to definition file")]
    definition_file: Option<PathBuf>,

    #[arg(short = 'x', long, help = "Execute renaming (disable DRY-RUN mode)")]
    execute: bool,

    #[arg(short, long, help = "Allow all rename confirmations")]
    yes: bool,

    #[arg(required = true, help = "Target directories or files")]
    path: Vec<PathBuf>,
    //#[arg(short, long, help = "Glob pattern of target directories or files")]
    //glob: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Editor Error: {0}")]
    Editor(#[from] editor::EditorError),

    #[error("Rename Error: {0}")]
    Rename(#[from] renamer::RenameError),

    #[error("Failed to create definition file")]
    CreateDefinitionFile(std::io::Error),

    #[error("Failed to open definition file")]
    OpenDefinitionFIle(std::io::Error),

    #[error("Failed to read definition files line")]
    ReadDefinitionFile(std::io::Error),

    #[error("Failed to write to definition file")]
    WriteToDefinitionFile(std::io::Error),
}

fn main() -> Result<(), AppError> {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    dbg!(&args);

    //let (exists, not_exists): (Vec<_>, Vec<_>) = args.path.iter().partition(|path| path.exists());
    //for path in not_exists {
    //    log::warn!("{} not exists", path.to_string_lossy());
    //}

    // Determine definition_file_path
    let definition_file_path = {
        if let Some(path) = args.definition_file {
            path
        } else {
            let file = tempfile::NamedTempFile::new()
                .map_err(|err| AppError::CreateDefinitionFile(err))?;
            file.path().to_path_buf()
        }
    };

    let origin = args.path.iter().collect::<Vec<&PathBuf>>();

    // Create and definition_file and write path entries
    save_to_definition_file(&definition_file_path, &origin)?;

    // Edit definition file with text editor
    println!("Launching text editor...");
    editor::execute_editor(args.editor.as_str(), &definition_file_path)?;

    // Load renamed entries from edited definition file
    let renamed = load_from_definition_file(&definition_file_path)?;

    renamer::rename(
        &origin,
        &renamed.iter().collect::<Vec<&PathBuf>>(),
        !args.execute,
        !args.yes,
    )?;

    Ok(())
}

fn save_to_definition_file<P: AsRef<Path>>(path: P, entries: &[P]) -> Result<(), AppError> {
    let mut file = std::fs::File::create(&path).map_err(AppError::CreateDefinitionFile)?;

    for entry in entries {
        writeln!(file, "{}", entry.as_ref().to_string_lossy())
            .map_err(AppError::WriteToDefinitionFile)?;
    }

    Ok(())
}

fn load_from_definition_file<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, AppError> {
    let mut entries = Vec::<PathBuf>::new();
    let file = File::open(path).map_err(AppError::OpenDefinitionFIle)?;
    let reader = BufReader::new(file);
    for result in reader.lines() {
        let line = result.map_err(AppError::ReadDefinitionFile)?;
        entries.push(PathBuf::from(line));
    }

    Ok(entries)
}

mod editor {
    use std::path::Path;
    use std::process::Command;
    use std::process::Stdio;

    #[derive(thiserror::Error, Debug)]
    pub enum EditorError {
        #[error("Failed to execute editor command")]
        Command(std::io::Error),

        #[error("Editor command was not successful")]
        Status(Option<i32>),
    }

    pub fn execute_editor<P: AsRef<Path>>(command: &str, path: P) -> Result<(), EditorError> {
        let status = Command::new(command)
            .args(path.as_ref().to_str())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(EditorError::Command)?;
        if !status.success() {
            return Err(EditorError::Status(status.code()));
        }

        Ok(())
    }
}

mod renamer {
    use std::path::{Path, PathBuf};

    #[derive(thiserror::Error, Debug)]
    pub enum RenameError {
        #[error(
            "Mismatch entries count (origin={0} renamed={1}): Entries must not be added or deleted"
        )]
        MismatchEntries(usize, usize),

        #[error("Failed to rename {0} -> {1}: {2}")]
        RenameFailure(PathBuf, PathBuf, std::io::Error),

        #[error("CLI Error: {0}")]
        Cli(#[from] crate::cli::CliError),
    }

    pub fn rename<P: AsRef<Path>>(
        origin: &[P],
        renamed: &[P],
        dry_run: bool,
        confirm: bool,
    ) -> Result<(), RenameError> {
        let changed = list_changed(origin, renamed)?;

        if changed.is_empty() {
            println!("Nothing to change");
            return Ok(());
        }

        for (src, dest) in changed {
            if dry_run {
                print!(
                    concat!(
                        //
                        "DRY RUN    {}\n",
                        "       --> {}\n",
                    ),
                    src.to_string_lossy(),
                    dest.to_string_lossy()
                );
                continue;
            } else {
                if confirm {
                    print!(
                        concat!(
                            //
                            "RENAME     {}\n",
                            "       --> {}\n",
                        ),
                        src.to_string_lossy(),
                        dest.to_string_lossy()
                    );
                    if !crate::cli::confirm("Accept the rename?")? {
                        continue;
                    }
                }

                std::fs::rename(&src, &dest)
                    .map_err(|err| RenameError::RenameFailure(src, dest, err))?;
            }
        }

        Ok(())
    }

    fn list_changed<P: AsRef<Path>>(
        origin: &[P],
        renamed: &[P],
    ) -> Result<Vec<(PathBuf, PathBuf)>, RenameError> {
        if origin.len() != renamed.len() {
            return Err(RenameError::MismatchEntries(origin.len(), renamed.len()));
        }

        let mut changed = vec![];

        for (src, dest) in origin.iter().zip(renamed.iter()) {
            if src.as_ref() != dest.as_ref() {
                changed.push((src.as_ref().to_path_buf(), dest.as_ref().to_path_buf()));
            }
        }

        Ok(changed)
    }
}

mod cli {
    use inquire;
    use std::io::Write;

    #[derive(Debug, thiserror::Error)]
    pub enum CliError {
        #[error("Prompt error: {0}")]
        Prompt(#[from] inquire::error::InquireError),

        #[error("failed read line: {0}")]
        ReadLine(std::io::Error),
    }

    pub fn confirm(prompt: &str) -> Result<bool, CliError> {
        let accept = inquire::Confirm::new(prompt).with_default(true).prompt()?;
        println!("");

        Ok(accept)
    }
}
