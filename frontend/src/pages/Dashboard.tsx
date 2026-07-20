import { api } from "../api/client";
import { useApi } from "../api/useApi";
import { DocumentCard, ErrorState, Icon, LinkButton, Loading, PageHeader } from "../components/Ui";

export function Dashboard() {
  const { data, error } = useApi(async () => {
    const [documents, tags, lint] = await Promise.all([api.documents(), api.tags(), api.lint()]);
    return { documents, tags, lint };
  }, []);
  if (error) return <ErrorState error={error} />;
  if (!data) return <Loading />;
  const kinds = {
    architecture: data.documents.filter((doc) => doc.kind === "architecture").length,
    decision: data.documents.filter((doc) => doc.kind === "decision").length,
    task: data.documents.filter((doc) => doc.kind === "task").length,
    research: data.documents.filter((doc) => doc.kind === "research").length,
  };
  const activeTasks = data.documents.filter(
    (doc) => doc.kind === "task" && ["todo", "in-progress"].includes(doc.status ?? ""),
  );

  return (
    <div className="page dashboard-page">
      <PageHeader kicker="Overview" title="Dashboard" />
      <section className="stat-grid">
        <LinkButton className="stat-card stat-primary" to="/documents">
          <span className="stat-icon">
            <Icon name="documents" />
          </span>
          <strong>{data.documents.length}</strong>
          <span>All documents</span>
          <small>Across the workspace</small>
        </LinkButton>
        <LinkButton className="stat-card" to="/architecture">
          <span className="stat-letter">A</span>
          <strong>{kinds.architecture}</strong>
          <span>Architecture</span>
          <small>How the system works</small>
        </LinkButton>
        <LinkButton className="stat-card" to="/decisions">
          <span className="stat-letter">D</span>
          <strong>{kinds.decision}</strong>
          <span>Decisions</span>
          <small>Why choices were made</small>
        </LinkButton>
        <LinkButton className="stat-card" to="/tasks">
          <span className="stat-letter">T</span>
          <strong>{kinds.task}</strong>
          <span>Tasks</span>
          <small>{activeTasks.length} currently open</small>
        </LinkButton>
        <LinkButton className="stat-card" to="/research">
          <span className="stat-letter">R</span>
          <strong>{kinds.research}</strong>
          <span>Research</span>
          <small>What was investigated</small>
        </LinkButton>
      </section>
      <div className="dashboard-columns">
        <section>
          <div className="section-heading">
            <div>
              <p className="kicker">Recently indexed</p>
              <h2>Document register</h2>
            </div>
            <LinkButton to="/documents">
              View all <span>→</span>
            </LinkButton>
          </div>
          <div className="document-list">
            {data.documents
              .slice(-5)
              .reverse()
              .map((doc, index) => (
                <DocumentCard document={doc} index={index} key={doc.id} />
              ))}
          </div>
        </section>
        <aside className="dashboard-rail">
          <section className={`health-card ${data.lint.errors > 0 ? "has-errors" : ""}`}>
            <div className="section-heading">
              <div>
                <p className="kicker">Repository health</p>
                <h2>Lint status</h2>
              </div>
              <span className="pulse-dot" />
            </div>
            <div className="health-score">
              <strong>{data.lint.errors === 0 ? "Clear" : data.lint.errors}</strong>
              <span>{data.lint.errors === 0 ? "No errors found" : "errors need attention"}</span>
            </div>
            <div className="health-breakdown">
              <span>
                <i className="error-dot" /> Errors <strong>{data.lint.errors}</strong>
              </span>
              <span>
                <i className="warning-dot" /> Warnings <strong>{data.lint.warnings}</strong>
              </span>
            </div>
            <LinkButton className="full-link" to="/lint">
              Open diagnostics <span>↗</span>
            </LinkButton>
          </section>
          <section className="popular-tags">
            <div className="section-heading">
              <div>
                <p className="kicker">Vocabulary</p>
                <h2>Active tags</h2>
              </div>
            </div>
            {data.tags.slice(0, 8).map((tag) => (
              <LinkButton key={tag.name} to={`/tag/${encodeURIComponent(tag.name)}`}>
                <span>#{tag.name}</span>
                <strong>{tag.count}</strong>
              </LinkButton>
            ))}
          </section>
        </aside>
      </div>
    </div>
  );
}
