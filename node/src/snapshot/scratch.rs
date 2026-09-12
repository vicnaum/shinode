//! Reclaim only exact files created by this import, never adopted source files.
use super::{download, journal::Identity, manifest::Plan};
use eyre::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Read, path::Path};

#[derive(Deserialize, Serialize)]
pub struct Ownership {
    identity: Identity,
    files: BTreeMap<String, Option<String>>,
}

fn inputs(plan: &Plan) -> Vec<String> {
    let mut paths = Vec::new();
    for chunk in &plan.chunks {
        paths.push(format!("archives/{}.tar.zst", chunk.name));
        paths.extend(
            chunk
                .files
                .iter()
                .map(|file| format!("chunks/{}/{}", chunk.name, file.path)),
        );
    }
    paths
}

fn fingerprint(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "scratch input is not a regular file"
    );
    let mut file = fs::File::open(path)?;
    let mut hash = blake3::Hasher::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().to_hex().to_string())
}

impl Ownership {
    fn load(root: &Path, plan: &Plan) -> Result<Self> {
        let identity = Identity::new(plan);
        let ownership = match fs::read(root.join("owned-inputs.json")) {
            Ok(bytes) => serde_json::from_slice::<Self>(&bytes)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self {
                identity: identity.clone(),
                files: BTreeMap::new(),
            },
            Err(error) => return Err(error.into()),
        };
        ensure!(
            ownership.identity == identity,
            "scratch ownership identity mismatch"
        );
        let allowed = inputs(plan);
        ensure!(
            ownership.files.keys().all(|path| allowed.contains(path)),
            "invalid scratch ownership path"
        );
        Ok(ownership)
    }

    pub fn prepare(root: &Path, plan: &Plan) -> Result<Self> {
        let mut ownership = Self::load(root, plan)?;
        for path in inputs(plan) {
            let needed = plan.chunks.iter().any(|chunk| {
                !super::manifest::chunk_path(root, chunk).exists()
                    && (path == format!("archives/{}.tar.zst", chunk.name)
                        || path.starts_with(&format!("chunks/{}/", chunk.name)))
            });
            if needed && !root.join(&path).try_exists()? {
                ownership.files.insert(path, None);
            }
        }
        download::atomic_json(&root.join("owned-inputs.json"), &ownership)?;
        Ok(ownership)
    }

    pub fn verified(&mut self, root: &Path, _plan: &Plan) -> Result<()> {
        for (path, digest) in &mut self.files {
            if digest.is_none() && root.join(path).try_exists()? {
                *digest = Some(fingerprint(&root.join(path))?);
            }
        }
        download::atomic_json(&root.join("owned-inputs.json"), self)
    }

    pub fn cleanup(root: &Path, plan: &Plan) -> Result<()> {
        let ownership = Self::load(root, plan)?;
        for (relative, expected) in &ownership.files {
            let path = root.join(relative);
            let Some(expected) = expected else {
                continue;
            };
            if !path.try_exists()? {
                continue;
            }
            // Preserve any file modified since publication, even under an owned name.
            if fingerprint(&path)? != *expected {
                continue;
            }
            fs::remove_file(&path)?;
            let mut parent = path.parent();
            while let Some(dir) = parent.filter(|dir| *dir != root) {
                match fs::remove_dir(dir) {
                    Ok(()) => parent = dir.parent(),
                    Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
                    Err(error) => return Err(error.into()),
                }
            }
        }
        // Keep the ownership journal for idempotent cleanup after interruption.
        crate::storage::durable::sync_directory(root)
    }
}
