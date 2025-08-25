use std::{
    env,
    fs::{self},
    path::PathBuf,
    sync::OnceLock,
};

use anyhow::{Result, anyhow};

// FUTURE: Should be able to make this an Arc for multi-threading
pub static CORPUS: OnceLock<Vec<String>> = OnceLock::new();

pub fn initialize_corpus() -> Result<()> {
    let corpus_dir = get_corpus_dir()?;
    let corpus = load_corpus(&corpus_dir)?;
    if corpus.is_empty() {
        return Err(anyhow!("No corpus entries in initialize_corpus"));
    }

    CORPUS
        .set(corpus)
        .map_err(|e| anyhow!(format!("Failed to initialize CORPUS: {:?}", e)))?;

    return Ok(());
}

pub fn get_corpus() -> &'static Vec<String> {
    return CORPUS.get().expect("CORPUS not initialized");
}

fn get_corpus_dir() -> Result<PathBuf> {
    let corpus_dir_parent: PathBuf = if cfg!(debug_assertions) {
        let cargo_root: String = env::var("CARGO_MANIFEST_DIR")?;
        cargo_root.into()
    } else {
        let exe_path = env::current_exe()?;
        if let Some(parent) = exe_path.parent() {
            parent.into()
        } else {
            return Err(anyhow!("No parent for {}", exe_path.display()));
        }
    };

    let corpus_dir: PathBuf = corpus_dir_parent.join("corpus");
    return Ok(corpus_dir);
}

// FUTURE: Will need to be updated with typing and weights for entries
fn load_corpus(corpus_dir: &PathBuf) -> Result<Vec<String>> {
    let corpus_content = match fs::read_dir(corpus_dir) {
        Ok(dir) => dir,
        Err(e) => {
            let err_string = format!("Unable to open {} -- {}", corpus_dir.display(), e);
            return Err(anyhow!(err_string));
        }
    };

    let mut corpus_files: Vec<String> = Vec::new();

    for entry in corpus_content {
        let file = entry?;

        let mut path = file.path();
        if path.is_file() {
            let contents = fs::read_to_string(&mut path)?;
            corpus_files.push(contents);
        }
    }

    if corpus_files.is_empty() {
        return Err(anyhow!("No corpus entries loaded"));
    }

    return Ok(corpus_files);
}
