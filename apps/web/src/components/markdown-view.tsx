import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";

const components: Components = {
  h1: ({ children }) => <h3 className="pt-2 text-xl font-semibold tracking-normal text-ink">{children}</h3>,
  h2: ({ children }) => <h3 className="pt-2 text-xl font-semibold tracking-normal text-ink">{children}</h3>,
  h3: ({ children }) => <h3 className="pt-2 text-lg font-semibold tracking-normal text-ink">{children}</h3>,
  h4: ({ children }) => <h4 className="pt-2 text-base font-semibold tracking-normal text-ink">{children}</h4>,
  h5: ({ children }) => <h5 className="pt-2 text-sm font-semibold tracking-normal text-ink">{children}</h5>,
  h6: ({ children }) => <h6 className="pt-2 text-sm font-semibold tracking-normal text-ink-muted">{children}</h6>,
  p: ({ children }) => <p>{children}</p>,
  ul: ({ className, children }) => (
    <ul className={`list-disc space-y-1 pl-5 ${className ?? ""}`.trim()}>{children}</ul>
  ),
  ol: ({ children }) => <ol className="list-decimal space-y-1 pl-5">{children}</ol>,
  li: ({ className, children }) => (
    <li className={`pl-1 ${className ?? ""}`.trim()}>{children}</li>
  ),
  a: ({ href, children }) => (
    <a
      className="font-medium text-action underline decoration-action/30 underline-offset-2 hover:text-action-strong hover:decoration-action"
      href={href}
      rel="noreferrer"
      target={href?.startsWith("http://") || href?.startsWith("https://") ? "_blank" : undefined}
    >
      {children}
    </a>
  ),
  table: ({ children }) => (
    <div className="overflow-x-auto rounded border border-slate-200">
      <table className="min-w-full border-collapse text-left text-sm">{children}</table>
    </div>
  ),
  thead: ({ children }) => <thead className="bg-surface-muted text-xs uppercase text-ink-muted">{children}</thead>,
  th: ({ children }) => <th className="border-b border-slate-200 px-3 py-2 font-semibold">{children}</th>,
  td: ({ children }) => <td className="border-b border-slate-100 px-3 py-2 align-top">{children}</td>,
  input: ({ checked, disabled, type }) => {
    if (type !== "checkbox") {
      return <input checked={checked} disabled={disabled} type={type} />;
    }
    return (
      <input
        checked={checked}
        className="mr-2 h-4 w-4 translate-y-0.5 rounded border-slate-300 accent-action"
        disabled
        readOnly
        type="checkbox"
      />
    );
  },
  code: ({ className, children }) => {
    const isBlock = /language-\w+/.test(className ?? "") || String(children).includes("\n");
    if (isBlock) {
      return (
        <code className={className} translate="no">
          {children}
        </code>
      );
    }
    return (
      <code className="rounded bg-surface-muted px-1.5 py-0.5 font-mono text-xs text-ink" translate="no">
        {children}
      </code>
    );
  },
  pre: ({ children }) => (
    <pre className="overflow-auto rounded bg-slate-950 p-3 text-xs leading-5 text-slate-100" translate="no">
      {children}
    </pre>
  ),
  blockquote: ({ children }) => (
    <blockquote className="border-l-4 border-slate-200 pl-4 text-ink-muted">{children}</blockquote>
  ),
};

export function MarkdownView({ markdown }: { markdown: string }) {
  return (
    <div className="space-y-3 text-sm leading-6 text-ink">
      <ReactMarkdown components={components} remarkPlugins={[remarkGfm]} skipHtml>
        {markdown}
      </ReactMarkdown>
    </div>
  );
}
