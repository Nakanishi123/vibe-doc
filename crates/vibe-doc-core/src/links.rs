//! Markdownリンクの抽出と逆リンクの生成を定義する。

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

/// 管理対象のMarkdown文書を指すリンク。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ManagedLink {
    pub source: PathBuf,
    pub target: PathBuf,
}

/// Markdown本文からインラインリンクの宛先を抽出する。
///
/// 文書間の参照だけを扱う小さな抽出器であり、Markdownのレンダリング用パーサーでは
/// ない。コードフェンス内、画像、エスケープされたブラケットを除外し、インライン
/// 形式の`[表示名](宛先)`だけを収集する。リンク先の存在確認と相対パスの解決は
/// [`managed_links`] が担うため、この関数は外部URLや存在しないパスもそのまま返す。
pub fn markdown_link_destinations(markdown: &str) -> Vec<String> {
    let mut destinations = Vec::new();
    let mut in_fenced_code = false;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fenced_code = !in_fenced_code;
            continue;
        }
        if in_fenced_code {
            continue;
        }
        let bytes = line.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'\\' {
                index += 2;
                continue;
            }
            if bytes[index] != b'[' || (index > 0 && bytes[index - 1] == b'!') {
                index += 1;
                continue;
            }
            let Some(close_label) = find_unescaped(bytes, index + 1, b']') else {
                break;
            };
            if bytes.get(close_label + 1) != Some(&b'(') {
                index = close_label + 1;
                continue;
            }
            let start = close_label + 2;
            let Some(end) = find_link_end(bytes, start) else {
                break;
            };
            let raw = line[start..end].trim();
            let destination = raw
                .strip_prefix('<')
                .and_then(|value| value.strip_suffix('>'))
                .unwrap_or(raw)
                .split_ascii_whitespace()
                .next()
                .unwrap_or_default();
            if !destination.is_empty() {
                destinations.push(destination.to_owned());
            }
            index = end + 1;
        }
    }
    destinations
}

/// 既知の文書パスを基準に、本文中のリンクを管理対象文書へのリンクへ限定する。
pub fn managed_links<'a>(
    source: &Path,
    markdown: &str,
    known_documents: impl IntoIterator<Item = &'a PathBuf>,
) -> Vec<ManagedLink> {
    let known_documents: BTreeSet<_> = known_documents
        .into_iter()
        .map(|path| normalize_path(path))
        .collect();
    markdown_link_destinations(markdown)
        .into_iter()
        .filter_map(|destination| {
            let path_part = destination.split(['#', '?']).next().unwrap_or_default();
            if path_part.is_empty() || is_external_destination(path_part) {
                return None;
            }
            let target = if Path::new(path_part).is_absolute() {
                normalize_path(Path::new(path_part))
            } else {
                normalize_path(
                    &source
                        .parent()
                        .unwrap_or_else(|| Path::new(""))
                        .join(path_part),
                )
            };
            known_documents.contains(&target).then(|| ManagedLink {
                source: normalize_path(source),
                target,
            })
        })
        .collect()
}

fn find_unescaped(bytes: &[u8], start: usize, needle: u8) -> Option<usize> {
    (start..bytes.len())
        .find(|&index| bytes[index] == needle && (index == 0 || bytes[index - 1] != b'\\'))
}

fn find_link_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut nested = 0;
    for (index, byte) in bytes.iter().enumerate().skip(start) {
        match byte {
            b'(' => nested += 1,
            b')' if nested == 0 => return Some(index),
            b')' => nested -= 1,
            _ => {}
        }
    }
    None
}

fn is_external_destination(destination: &str) -> bool {
    destination.starts_with("//")
        || destination.contains("://")
        || destination.starts_with("mailto:")
        || destination.starts_with("tel:")
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component)
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{ManagedLink, managed_links, markdown_link_destinations};
    use std::path::PathBuf;
    #[test]
    fn extracts_inline_links_but_not_images_or_code() {
        assert_eq!(
            markdown_link_destinations(
                "[one](one.md#heading) ![image](image.md)\n```md\n[ignored](ignored.md)\n```\n[two](<two.md>)"
            ),
            ["one.md#heading", "two.md"]
        );
    }
    #[test]
    fn resolves_only_known_local_documents() {
        let source = PathBuf::from("docs/tasks/todo/current.md");
        let known = [source.clone(), PathBuf::from("docs/architecture/model.md")];
        assert_eq!(
            managed_links(
                &source,
                "[model](../../architecture/model.md) [web](https://example.com)",
                known.iter()
            ),
            [ManagedLink {
                source,
                target: PathBuf::from("docs/architecture/model.md")
            }]
        );
    }
}
