import type { Parent, PhrasingContent, Root } from "mdast";

/**
 * 本文テキスト中の文書ID（例: DEC-0007, TASK-0012, ARCH-0003）にマッチするパターン。
 * 「大文字英数字のプレフィックス + ハイフン + 数字」を候補として抽出し、
 * 実在する文書IDかどうかは knownIds との照合で判定する。
 */
const DOCUMENT_ID_PATTERN = /\b[A-Z][A-Z0-9]*-\d+\b/g;

/** すでにリンクである要素の内側は変換しない。 */
const SKIPPED_PARENTS = new Set(["link", "linkReference"]);

export interface DocumentIdLinkOptions {
  knownIds: ReadonlySet<string>;
}

/**
 * remarkプラグイン: テキストノード中の既知の文書IDを
 * `/documents/{id}` へのリンクノードに置き換える。
 * コードブロック・インラインコード・既存リンク内は対象外。
 */
export function remarkDocumentIdLinks({ knownIds }: DocumentIdLinkOptions) {
  return (tree: Root) => {
    if (knownIds.size > 0) {
      linkifyChildren(tree, knownIds);
    }
  };
}

function linkifyChildren(parent: Parent, knownIds: ReadonlySet<string>) {
  const replaced: typeof parent.children = [];
  for (const child of parent.children) {
    if (child.type === "text") {
      replaced.push(...linkifyText(child.value, knownIds));
      continue;
    }
    if ("children" in child && !SKIPPED_PARENTS.has(child.type)) {
      linkifyChildren(child, knownIds);
    }
    replaced.push(child);
  }
  parent.children = replaced;
}

function linkifyText(value: string, knownIds: ReadonlySet<string>): PhrasingContent[] {
  const nodes: PhrasingContent[] = [];
  let cursor = 0;
  for (const match of value.matchAll(DOCUMENT_ID_PATTERN)) {
    const id = match[0];
    if (!knownIds.has(id)) continue;
    if (match.index > cursor) {
      nodes.push({ type: "text", value: value.slice(cursor, match.index) });
    }
    nodes.push({
      type: "link",
      url: `/documents/${encodeURIComponent(id)}`,
      children: [{ type: "text", value: id }],
    });
    cursor = match.index + id.length;
  }
  if (nodes.length === 0) {
    return [{ type: "text", value }];
  }
  if (cursor < value.length) {
    nodes.push({ type: "text", value: value.slice(cursor) });
  }
  return nodes;
}
