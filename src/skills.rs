//! `qorx skills share`: a skill folder (a folder with a SKILL.md) found in one AI tool's skills
//! folder is copied into the others, so a skill added in Claude Code, Codex, Antigravity or
//! OpenCode works in all of them. Run it whenever you add or change a skill.
//!
//! Qorx writes only folders it marks as its own copies (a `.qorx-shared.json` inside) and never
//! changes your own skill folders. Two different skills with the same name stay where they are.
//! A copy you edit becomes yours. A copy whose original is gone is removed. OpenCode reads
//! `~/.claude/skills` itself, so its own folder is only read. `qorx skills unshare` removes
//! every copy Qorx made and nothing else.
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const MARKER: &str = ".qorx-shared.json";
const SKIP_DIRS: [&str; 4] = [".git", "node_modules", "__pycache__", ".venv"];
const MAX_FILES: usize = 500;
const MAX_BYTES: u64 = 20 * 1024 * 1024;

pub struct SkillsDir {
    pub tool: &'static str,
    pub dir: PathBuf,
    pub write: bool,
}

#[derive(Default, Debug, Serialize)]
pub struct ShareReport {
    pub copied: Vec<String>,
    pub updated: Vec<String>,
    pub removed: Vec<String>,
    pub adopted: Vec<String>,
    pub same_name_different_skill: Vec<String>,
    pub too_big: Vec<String>,
}

fn home() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn env_dir(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// The skills folders of the AI tools set up on this computer. A tool counts when its own folder exists.
pub fn tool_dirs() -> Vec<SkillsDir> {
    let Some(home) = home() else {
        return Vec::new();
    };
    let mut dirs = Vec::new();
    let claude = env_dir("CLAUDE_CONFIG_DIR").unwrap_or_else(|| home.join(".claude"));
    if claude.is_dir() {
        dirs.push(SkillsDir {
            tool: "Claude Code",
            dir: claude.join("skills"),
            write: true,
        });
    }
    let codex = env_dir("CODEX_HOME").unwrap_or_else(|| home.join(".codex"));
    if codex.is_dir() {
        dirs.push(SkillsDir {
            tool: "Codex",
            dir: codex.join("skills"),
            write: true,
        });
    }
    let antigravity = home.join(".gemini").join("config");
    if antigravity.is_dir() {
        dirs.push(SkillsDir {
            tool: "Antigravity",
            dir: antigravity.join("skills"),
            write: true,
        });
    }
    let opencode = env_dir("XDG_CONFIG_HOME")
        .unwrap_or_else(|| home.join(".config"))
        .join("opencode");
    dirs.push(SkillsDir {
        tool: "OpenCode",
        dir: opencode.join("skills"),
        write: false,
    });
    dirs
}

struct Original {
    tool: &'static str,
    dir: PathBuf,
    hash: String,
}

/// One round: adopt edited copies, copy every skill to the tools that lack it, refresh stale
/// copies and remove copies whose original is gone.
pub fn share(dirs: &[SkillsDir]) -> Result<ShareReport> {
    let mut report = ShareReport::default();
    let mut originals: BTreeMap<String, Vec<Original>> = BTreeMap::new();
    let mut copies: Vec<(usize, String, PathBuf, String)> = Vec::new();
    for (index, tool) in dirs.iter().enumerate() {
        for (name, dir) in skill_folders(&tool.dir) {
            let Some(hash) = folder_hash(&dir) else {
                report.too_big.push(format!("{} in {}", name, tool.tool));
                continue;
            };
            match read_marker(&dir) {
                Some(marked) if marked == hash => copies.push((index, name, dir, hash)),
                Some(_) => {
                    // You changed Qorx's copy: from now on it is yours.
                    fs::remove_file(dir.join(MARKER))?;
                    report.adopted.push(format!("{} in {}", name, tool.tool));
                    originals.entry(name).or_default().push(Original {
                        tool: tool.tool,
                        dir,
                        hash,
                    });
                }
                None => originals.entry(name).or_default().push(Original {
                    tool: tool.tool,
                    dir,
                    hash,
                }),
            }
        }
    }

    let mut sources: BTreeMap<String, &Original> = BTreeMap::new();
    for (name, found) in &originals {
        if found.iter().all(|original| original.hash == found[0].hash) {
            sources.insert(name.clone(), &found[0]);
        } else {
            report.same_name_different_skill.push(name.clone());
        }
    }

    for (index, name, dir, hash) in &copies {
        if report.same_name_different_skill.contains(name) {
            continue;
        }
        match sources.get(name) {
            Some(source) if &source.hash == hash => {}
            Some(source) => {
                copy_skill(&source.dir, dir, &source.hash, source.tool)?;
                report
                    .updated
                    .push(format!("{} in {}", name, dirs[*index].tool));
            }
            None => {
                fs::remove_dir_all(dir)?;
                report
                    .removed
                    .push(format!("{} from {}", name, dirs[*index].tool));
            }
        }
    }

    for (name, source) in &sources {
        for tool in dirs.iter().filter(|tool| tool.write) {
            let target = tool.dir.join(name);
            if target.exists() {
                continue;
            }
            copy_skill(&source.dir, &target, &source.hash, source.tool)?;
            report
                .copied
                .push(format!("{} from {} to {}", name, source.tool, tool.tool));
        }
    }
    Ok(report)
}

/// Remove every copy Qorx made, in every tool, and nothing else. Returns how many it removed.
pub fn unshare(dirs: &[SkillsDir]) -> Result<usize> {
    let mut removed = 0;
    for tool in dirs.iter().filter(|tool| tool.write) {
        for (_, dir) in skill_folders(&tool.dir) {
            if dir.join(MARKER).is_file() {
                fs::remove_dir_all(&dir)?;
                removed += 1;
            }
        }
    }
    Ok(removed)
}

fn skill_folders(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| Some((entry.file_name().into_string().ok()?, entry.path())))
        .filter(|(name, dir)| !name.starts_with('.') && dir.join("SKILL.md").is_file())
        .collect();
    found.sort();
    found
}

fn skill_files(root: &Path) -> Option<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    let mut bytes = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).ok()?.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let kind = entry.file_type().ok()?;
            if kind.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(path);
                }
            } else if kind.is_file() && !(dir == root && name == MARKER) {
                bytes += entry.metadata().ok()?.len();
                let relative = path
                    .strip_prefix(root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((relative, path));
                if files.len() > MAX_FILES || bytes > MAX_BYTES {
                    return None;
                }
            }
        }
    }
    files.sort();
    Some(files)
}

fn folder_hash(root: &Path) -> Option<String> {
    let mut hasher = Sha256::new();
    for (relative, path) in skill_files(root)? {
        let content = fs::read(&path).ok()?;
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(&content);
    }
    Some(format!("{:x}", hasher.finalize()))
}

fn read_marker(dir: &Path) -> Option<String> {
    let text = fs::read_to_string(dir.join(MARKER)).ok()?;
    serde_json::from_str::<Value>(&text)
        .ok()?
        .get("hash")?
        .as_str()
        .map(str::to_string)
}

fn copy_skill(source: &Path, target: &Path, hash: &str, tool: &str) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("no parent for {}", target.display()))?;
    fs::create_dir_all(parent)?;
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let staging = parent.join(format!(".qorx-copying-{name}"));
    let _ = fs::remove_dir_all(&staging);
    let files =
        skill_files(source).ok_or_else(|| anyhow!("{} is too big to share", source.display()))?;
    for (relative, path) in files {
        let destination = staging.join(&relative);
        if let Some(folder) = destination.parent() {
            fs::create_dir_all(folder)?;
        }
        fs::copy(&path, &destination)?;
    }
    let marker = json!({"from": tool, "source": source.display().to_string(), "hash": hash,
        "note": "Copied by Qorx from another AI tool. Edit it and it becomes yours; Qorx then leaves it alone."});
    fs::write(staging.join(MARKER), serde_json::to_string_pretty(&marker)?)?;
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    fs::rename(&staging, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(root: &Path, name: &str, body: &str) -> PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("scripts")).unwrap();
        fs::write(
            dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: test\n---\n{body}\n"),
        )
        .unwrap();
        fs::write(dir.join("scripts").join("run.py"), "print('hi')\n").unwrap();
        dir
    }

    #[test]
    fn skills_spread_to_every_tool_and_follow_their_original() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("qorx-skills-{nonce}"));
        let dirs = vec![
            SkillsDir {
                tool: "Claude Code",
                dir: root.join("claude"),
                write: true,
            },
            SkillsDir {
                tool: "Codex",
                dir: root.join("codex"),
                write: true,
            },
            SkillsDir {
                tool: "Antigravity",
                dir: root.join("antigravity"),
                write: true,
            },
            SkillsDir {
                tool: "OpenCode",
                dir: root.join("opencode"),
                write: false,
            },
        ];
        let lazy = skill(&dirs[0].dir, "lazy", "be lazy");
        skill(&dirs[1].dir, "motion", "make videos");
        skill(&dirs[3].dir, "from-opencode", "opencode only");
        skill(&dirs[0].dir, "twin", "one");
        skill(&dirs[1].dir, "twin", "two");
        skill(&dirs[0].dir, "same", "alike");
        skill(&dirs[1].dir, "same", "alike");
        fs::create_dir_all(dirs[0].dir.join("synced").join("abc").join("docs")).unwrap();
        fs::write(
            dirs[0]
                .dir
                .join("synced")
                .join("abc")
                .join("docs")
                .join("SKILL.md"),
            "x",
        )
        .unwrap();

        let report = share(&dirs).unwrap();
        for tool in &dirs[..3] {
            for name in ["lazy", "motion", "from-opencode", "same"] {
                assert!(
                    tool.dir.join(name).join("SKILL.md").is_file(),
                    "{name} in {}",
                    tool.tool
                );
            }
            assert!(!tool.dir.join("synced").join("SKILL.md").exists());
        }
        assert!(
            !dirs[3].dir.join("lazy").exists(),
            "OpenCode's own folder is only read"
        );
        assert!(!dirs[2].dir.join("twin").exists());
        assert_eq!(report.same_name_different_skill, vec!["twin".to_string()]);
        assert!(dirs[1]
            .dir
            .join("lazy")
            .join("scripts")
            .join("run.py")
            .is_file());
        assert!(dirs[1].dir.join("lazy").join(MARKER).is_file());
        assert!(!lazy.join(MARKER).exists(), "the original is never marked");

        let again = share(&dirs).unwrap();
        assert!(again.copied.is_empty() && again.updated.is_empty() && again.removed.is_empty());

        fs::write(
            lazy.join("SKILL.md"),
            "---\nname: lazy\ndescription: test\n---\nlazier\n",
        )
        .unwrap();
        let updated = share(&dirs).unwrap();
        assert_eq!(updated.updated.len(), 2);
        assert!(
            fs::read_to_string(dirs[2].dir.join("lazy").join("SKILL.md"))
                .unwrap()
                .contains("lazier")
        );

        fs::write(
            dirs[2].dir.join("motion").join("SKILL.md"),
            "---\nname: motion\ndescription: t\n---\nmine\n",
        )
        .unwrap();
        let adopted = share(&dirs).unwrap();
        assert_eq!(adopted.adopted, vec!["motion in Antigravity".to_string()]);
        assert!(!dirs[2].dir.join("motion").join(MARKER).exists());
        assert!(
            dirs[0].dir.join("motion").join(MARKER).is_file(),
            "the other copy is left as it was"
        );

        fs::remove_dir_all(&lazy).unwrap();
        let removed = share(&dirs).unwrap();
        assert_eq!(removed.removed.len(), 2);

        assert!(unshare(&dirs).unwrap() > 0);
        assert!(!dirs[0].dir.join("from-opencode").exists());
        assert!(dirs[3].dir.join("from-opencode").is_dir());
        assert!(dirs[0].dir.join("twin").is_dir() && dirs[1].dir.join("twin").is_dir());
        assert!(
            dirs[2].dir.join("motion").is_dir(),
            "an adopted copy is your own"
        );
        let _ = fs::remove_dir_all(root);
    }
}
