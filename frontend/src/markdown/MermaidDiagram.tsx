import { useEffect, useRef, useState } from "react";
import type { RenderResult } from "mermaid";
import { useTheme } from "../components/ThemeContext";
import { loadMermaid, renderMermaid } from "./mermaid";

type RenderState =
  | { kind: "loading" }
  | ({ kind: "rendered" } & RenderResult)
  | { kind: "load-error" }
  | { kind: "syntax-error" };

let diagramSequence = 0;

function nextDiagramId(): string {
  diagramSequence += 1;
  return `vibe-doc-mermaid-${diagramSequence}`;
}

export function MermaidDiagram({ source }: { source: string }) {
  const theme = useTheme();
  const [state, setState] = useState<RenderState>({ kind: "loading" });
  const diagramRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let active = true;
    setState({ kind: "loading" });

    loadMermaid().then(
      (mermaid) => {
        renderMermaid(mermaid, nextDiagramId(), source, theme).then(
          (result) => {
            if (active) setState({ kind: "rendered", ...result });
          },
          () => {
            if (active) setState({ kind: "syntax-error" });
          },
        );
      },
      () => {
        if (active) setState({ kind: "load-error" });
      },
    );

    return () => {
      active = false;
    };
  }, [source, theme]);

  useEffect(() => {
    if (state.kind === "rendered" && diagramRef.current && state.bindFunctions) {
      state.bindFunctions(diagramRef.current);
    }
  }, [state]);

  if (state.kind === "rendered") {
    return (
      <figure className="mermaid-diagram">
        <div ref={diagramRef} dangerouslySetInnerHTML={{ __html: state.svg }} />
      </figure>
    );
  }

  const errorMessage =
    state.kind === "load-error"
      ? "Mermaidを読み込めなかったため、元のコードを表示しています。"
      : state.kind === "syntax-error"
        ? "Mermaid記法を描画できなかったため、元のコードを表示しています。"
        : undefined;

  return (
    <figure className="mermaid-fallback" aria-busy={state.kind === "loading"}>
      <pre>
        <code className="language-mermaid">{source}</code>
      </pre>
      {errorMessage && <figcaption role="status">{errorMessage}</figcaption>}
    </figure>
  );
}
