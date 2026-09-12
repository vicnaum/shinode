use super::args::SnapshotArgs;
use eyre::{ensure, eyre, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

pub const COMPONENTS: [&str; 3] = ["headers", "transactions", "receipts"];

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub block: u64,
    chain_id: u64,
    storage_version: u64,
    pub reth_version: String,
    base_url: String,
    components: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Component {
    blocks_per_file: u64,
    total_blocks: u64,
    chunk_files: Vec<String>,
    chunk_sizes: Vec<u64>,
    chunk_decompressed_sizes: Vec<u64>,
    chunk_output_files: Vec<Vec<OutputFile>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutputFile {
    pub path: String,
    pub size: u64,
    pub blake3: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Chunk {
    pub name: String,
    pub component: String,
    pub start: u64,
    pub end: u64,
    pub url: String,
    pub compressed: u64,
    pub extracted: u64,
    pub files: Vec<OutputFile>,
}

#[derive(Debug, Serialize)]
pub struct Plan {
    pub manifest_sha256: String,
    pub snapshot_height: u64,
    pub producer: String,
    pub requested_range: [u64; 2],
    pub dependency_range: [u64; 2],
    pub dependency_reason: &'static str,
    pub supported_format: &'static str,
    pub compressed_bytes: u64,
    pub extracted_bytes: u64,
    pub chunks: Vec<Chunk>,
}

pub fn read(args: &SnapshotArgs) -> Result<(Vec<u8>, Plan)> {
    ensure!(
        fs::metadata(&args.manifest)?.len() <= 16 * 1024 * 1024,
        "manifest exceeds 16 MiB"
    );
    let raw = fs::read(&args.manifest)?;
    let plan = inspect(&raw, args)?;
    Ok((raw, plan))
}

pub fn inspect(raw: &[u8], args: &SnapshotArgs) -> Result<Plan> {
    let digest = format!("{:x}", Sha256::digest(raw));
    ensure!(digest == args.manifest_sha256, "manifest SHA-256 mismatch");
    let manifest: Manifest = serde_json::from_slice(raw)?;
    ensure!(
        manifest.chain_id == 8453 && manifest.storage_version == 2,
        "expected Base mainnet V2 snapshot"
    );
    ensure!(
        manifest.reth_version == "2.5.2-dev (76a8261)",
        "unsupported snapshot producer {}; expected the verified 2.5.2-dev (76a8261) format",
        manifest.reth_version
    );
    ensure!(
        args.start_block <= args.end_block,
        "requested end precedes start"
    );
    ensure!(
        args.end_block <= manifest.block,
        "requested range is outside snapshot"
    );
    let base = reqwest::Url::parse(&manifest.base_url)?;
    ensure!(
        base.scheme() == "https"
            && base.has_host()
            && base.username().is_empty()
            && base.password().is_none()
            && base.query().is_none()
            && base.fragment().is_none(),
        "snapshot base URL must use HTTPS without credentials/query/fragment"
    );
    let mut chunks = Vec::new();
    let total = manifest
        .block
        .checked_add(1)
        .ok_or_else(|| eyre!("snapshot height overflow"))?;
    for name in COMPONENTS {
        let component: Component = serde_json::from_value(
            manifest
                .components
                .get(name)
                .ok_or_else(|| eyre!("missing {name} component"))?
                .clone(),
        )?;
        ensure!(
            component.blocks_per_file > 0 && component.total_blocks == total,
            "{name}: pruned history or invalid chunk width"
        );
        let count = usize::try_from(total.div_ceil(component.blocks_per_file))?;
        ensure!(
            [
                component.chunk_files.len(),
                component.chunk_sizes.len(),
                component.chunk_decompressed_sizes.len(),
                component.chunk_output_files.len()
            ]
            .iter()
            .all(|len| *len == count),
            "{name}: incomplete chunk metadata"
        );
        for index in 0..=usize::try_from(args.end_block / component.blocks_per_file)? {
            chunks.push(select_chunk(&manifest.base_url, name, &component, index)?);
        }
    }
    chunks.sort_by_key(|chunk| (chunk.start, chunk.component.clone()));
    let sum = |extract: bool| {
        chunks.iter().try_fold(0u64, |total, c| {
            total
                .checked_add(if extract { c.extracted } else { c.compressed })
                .ok_or_else(|| eyre!("input size overflow"))
        })
    };
    Ok(Plan {
        manifest_sha256: digest,
        snapshot_height: manifest.block,
        producer: manifest.reth_version,
        requested_range: [args.start_block, args.end_block],
        dependency_range: [0, args.end_block],
        dependency_reason: "sequential transaction-boundary reconstruction from genesis",
        supported_format: "verified producer; pre-Canyon bodies only; fresh output directory",
        compressed_bytes: sum(false)?,
        extracted_bytes: sum(true)?,
        chunks,
    })
}

fn select_chunk(base: &str, name: &str, c: &Component, index: usize) -> Result<Chunk> {
    let start = (index as u64)
        .checked_mul(c.blocks_per_file)
        .ok_or_else(|| eyre!("chunk range overflow"))?;
    let end = start
        .checked_add(c.blocks_per_file - 1)
        .ok_or_else(|| eyre!("chunk range overflow"))?;
    let stem = format!("{name}-{start}-{end}");
    let path = &c.chunk_files[index];
    let (parent, filename) = path
        .split_once('/')
        .ok_or_else(|| eyre!("invalid chunk path"))?;
    ensure!(
        (parent == "static_files"
            || (!parent.is_empty() && parent.bytes().all(|b| b.is_ascii_digit())))
            && filename == format!("{stem}.tar.zst"),
        "unexpected chunk path: {path}"
    );
    let files = c.chunk_output_files[index].clone();
    let prefix = format!("static_files/static_file_{name}_{start}_{end}");
    ensure!(files.len() == 3, "{stem}: expected data/conf/off files");
    for suffix in ["", ".conf", ".off"] {
        ensure!(
            files
                .iter()
                .filter(|f| f.path == format!("{prefix}{suffix}"))
                .count()
                == 1,
            "{stem}: invalid output paths"
        );
    }
    let mut extracted = 0u64;
    for file in &files {
        ensure!(
            file.blake3.len() == 64
                && file
                    .blake3
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "invalid BLAKE3 checksum"
        );
        extracted = extracted
            .checked_add(file.size)
            .ok_or_else(|| eyre!("output size overflow"))?;
    }
    ensure!(
        extracted == c.chunk_decompressed_sizes[index] && c.chunk_sizes[index] > 0,
        "inconsistent archive sizes"
    );
    Ok(Chunk {
        name: stem,
        component: name.to_owned(),
        start,
        end,
        url: format!("{}/{}", base.trim_end_matches('/'), path),
        compressed: c.chunk_sizes[index],
        extracted,
        files,
    })
}

pub fn validate_layout(plan: &Plan) -> Result<()> {
    for header in plan.chunks.iter().filter(|c| c.component == "headers") {
        for name in ["transactions", "receipts"] {
            ensure!(
                plan.chunks
                    .iter()
                    .any(|c| c.component == name && c.start == header.start && c.end == header.end),
                "reader requires aligned component block ranges; unsupported {name} layout"
            );
        }
    }
    Ok(())
}

pub fn chunk_path(root: &Path, chunk: &Chunk) -> std::path::PathBuf {
    root.join("chunks").join(&chunk.name)
}
