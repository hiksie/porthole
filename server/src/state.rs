use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct SharedFolder {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct AppState {
    folders: watch::Receiver<Arc<Vec<SharedFolder>>>,
}

impl AppState {
    #[cfg(test)]
    pub fn new(folders: Vec<PathBuf>) -> Self {
        FolderHandle::new(&folders).1
    }

    pub fn folders(&self) -> Arc<Vec<SharedFolder>> {
        self.folders.borrow().clone()
    }

    #[cfg(test)]
    pub fn root_ids(&self) -> Vec<String> {
        self.folders.borrow().iter().map(|f| f.id.clone()).collect()
    }

    pub fn root_by_id(&self, id: &str) -> Option<SharedFolder> {
        self.folders.borrow().iter().find(|f| f.id == id).cloned()
    }
}

pub struct FolderHandle {
    tx: watch::Sender<Arc<Vec<SharedFolder>>>,
}

impl FolderHandle {
    pub fn new(folders: &[PathBuf]) -> (Self, AppState) {
        let (tx, rx) = watch::channel(Arc::new(build(folders)));
        (Self { tx }, AppState { folders: rx })
    }

    pub fn update(&self, folders: &[PathBuf]) {
        self.tx.send_replace(Arc::new(build(folders)));
    }
}

fn build(folders: &[PathBuf]) -> Vec<SharedFolder> {
    let mut seen = Vec::new();
    let mut roots = Vec::new();

    for path in folders {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());

        if seen.contains(&canonical) {
            continue;
        }

        let name = canonical
            .file_name()
            .or_else(|| path.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| canonical.to_string_lossy().into_owned());

        roots.push(SharedFolder {
            id: folder_id(&canonical),
            name,
            path: canonical.clone(),
        });

        seen.push(canonical);
    }

    roots
}

fn folder_id(canonical: &Path) -> String {
    struct Fnv1a(u64);

    impl Hasher for Fnv1a {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, bytes: &[u8]) {
            for &b in bytes {
                self.0 ^= u64::from(b);
                self.0 = self.0.wrapping_mul(0x0000_0100_0000_01B3);
            }
        }
    }

    let mut hasher = Fnv1a(0xcbf2_9ce4_8422_2325);
    canonical.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
