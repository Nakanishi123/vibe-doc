import { useEffect, useState } from "react";
import { api } from "../api/client";
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
  const [reloading, setReloading] = useState(false);
  const [reloadError, setReloadError] = useState("");
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    const saved = window.localStorage.getItem("vibe-doc-theme");
    if (saved === "light" || saved === "dark") return saved;
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  });

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("vibe-doc-theme", theme);
  }, [theme]);

  async function reloadDocuments() {
    setReloading(true);
    setReloadError("");
    try {
      await api.reload();
      window.location.reload();
    } catch (error) {
      setReloading(false);
      setReloadError(error instanceof Error ? error.message : String(error));
    }
  }

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
        <div className="sidebar-footer">
          <div className="read-only-note">
            <span>●</span>
            <div>
              <strong>Read-only</strong>
              <small>Source files stay untouched</small>
            </div>
          </div>
          <button
            aria-label="Reload documents from disk"
            className="sidebar-action reload-button"
            disabled={reloading}
            onClick={reloadDocuments}
            type="button"
          >
            <Icon name="reload" />
            {reloading ? "Reloading…" : "Reload"}
          </button>
          {reloadError && (
            <p aria-live="polite" className="reload-error" role="status">
              {reloadError}
            </p>
          )}
          <button
            aria-label={`Switch to ${theme === "light" ? "dark" : "light"} mode`}
            className="sidebar-action theme-toggle"
            onClick={() => setTheme(theme === "light" ? "dark" : "light")}
            type="button"
          >
            <span aria-hidden="true">{theme === "light" ? "☾" : "☀"}</span>
            {theme === "light" ? "Dark mode" : "Light mode"}
          </button>
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
