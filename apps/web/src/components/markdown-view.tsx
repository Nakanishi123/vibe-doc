import { useMemo } from "react";

import { markdownToBlocks } from "../lib/markdown";

export function MarkdownView({ markdown }: { markdown: string }) {
  const blocks = useMemo(() => markdownToBlocks(markdown), [markdown]);
  return (
    <div className="space-y-3 text-sm leading-6 text-ink">
      {blocks.map((block, index) => {
        if (block.type === "heading") {
          const Heading = `h${Math.min(block.level + 2, 4)}` as "h3" | "h4";
          return <Heading className="pt-2 text-lg font-semibold tracking-normal" key={index}>{block.text}</Heading>;
        }
        if (block.type === "code") {
          return <pre className="overflow-auto rounded bg-slate-950 p-3 text-xs text-slate-100" key={index} translate="no"><code>{block.text}</code></pre>;
        }
        if (block.type === "list") {
          return <ul className="list-disc space-y-1 pl-5" key={index}>{block.items.map((item, itemIndex) => <li key={itemIndex}>{item}</li>)}</ul>;
        }
        return <p key={index}>{block.text}</p>;
      })}
    </div>
  );
}
