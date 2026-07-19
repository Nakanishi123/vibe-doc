import { isValidElement, type ReactNode } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { DocumentSummary } from "../types";
import { navigate } from "../routes/navigation";
import { MermaidDiagram } from "./MermaidDiagram";

function MermaidPre({ children }: { children?: ReactNode }) {
  if (
    isValidElement<{ className?: string; children?: ReactNode }>(children) &&
    children.props.className?.split(/\s+/).includes("language-mermaid")
  ) {
    return <MermaidDiagram source={String(children.props.children ?? "").replace(/\n$/, "")} />;
  }

  return <pre>{children}</pre>;
}

export function MarkdownBody({
  source,
  linkedDocuments,
}: {
  source: string;
  linkedDocuments: DocumentSummary[];
}) {
  return (
    <div className="markdown-body">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          pre: MermaidPre,
          a({ href = "", children, ...props }) {
            const path = decodeURIComponent(href.split(/[?#]/)[0] ?? "");
            const filename = path.split("/").pop();
            const target = linkedDocuments.find((document) =>
              document.path.endsWith(`/${filename}`),
            );
            if (target) {
              return (
                <a
                  {...props}
                  href={`/documents/${encodeURIComponent(target.id)}`}
                  onClick={(event) => {
                    event.preventDefault();
                    navigate(`/documents/${encodeURIComponent(target.id)}`);
                  }}
                >
                  {children}
                </a>
              );
            }
            const external = /^https?:\/\//.test(href);
            return (
              <a
                {...props}
                href={href}
                rel={external ? "noreferrer" : undefined}
                target={external ? "_blank" : undefined}
              >
                {children}
              </a>
            );
          },
        }}
      >
        {source}
      </ReactMarkdown>
    </div>
  );
}
