import { useState } from "react";
import { Icon, LinkButton } from "./Ui";

const nav = [
  ["/", "dashboard", "Dashboard"],
  ["/documents", "documents", "Documents"],
  ["/decisions", "decisions", "Decisions"],
  ["/tasks", "tasks", "Tasks"],
  ["/tags", "tags", "Tags"],
  ["/links", "links", "Links"],
  ["/lint", "lint", "Lint"],
] as const;

export function Layout({ pathname, children }: { pathname: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="app-shell">
      <button
        aria-label="Toggle navigation"
        className="menu-toggle"
        onClick={() => setOpen(!open)}
        type="button"
      >
        ☰
      </button>
      <aside className={`sidebar ${open ? "sidebar-open" : ""}`}>
        <LinkButton className="brand" to="/">
          <span className="brand-glyph">V</span>
          <span>
            <strong>vibe-doc</strong>
            <small>DOCUMENT INTELLIGENCE</small>
          </span>
        </LinkButton>
        <nav aria-label="Primary navigation">
          <p className="nav-label">Workspace</p>
          {nav.map(([path, icon, label]) => {
            const active = path === "/" ? pathname === "/" : pathname.startsWith(path);
            return (
              <LinkButton className={`nav-item ${active ? "active" : ""}`} key={path} to={path}>
                <Icon name={icon} />
                <span>{label}</span>
                {active && <i />}
              </LinkButton>
            );
          })}
        </nav>
        <div className="read-only-note">
          <span>●</span>
          <div>
            <strong>Read-only</strong>
            <small>Source files stay untouched</small>
          </div>
        </div>
      </aside>
      {open && (
        <button
          aria-label="Close navigation"
          className="sidebar-scrim"
          onClick={() => setOpen(false)}
          type="button"
        />
      )}
      <main className="main-content">{children}</main>
    </div>
  );
}
