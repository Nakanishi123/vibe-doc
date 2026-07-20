import { Layout } from "./components/Layout";
import { EmptyState, LinkButton } from "./components/Ui";
import { Dashboard } from "./pages/Dashboard";
import { DocumentDetail } from "./pages/DocumentDetail";
import { DocumentList } from "./pages/DocumentList";
import { Links } from "./pages/Links";
import { LintPage } from "./pages/Lint";
import { TagDetailPage, Tags } from "./pages/Tags";
import { usePathname } from "./routes/navigation";
import "./styles.css";

function CurrentPage({ pathname }: { pathname: string }) {
  if (pathname === "/") return <Dashboard />;
  if (pathname === "/documents") return <DocumentList key="documents" mode="documents" />;
  if (pathname === "/architecture")
    return <DocumentList key="architecture" mode="architecture" />;
  if (pathname === "/decisions") return <DocumentList key="decisions" mode="decisions" />;
  if (pathname === "/tasks") return <DocumentList key="tasks" mode="tasks" />;
  if (pathname === "/research") return <DocumentList key="research" mode="research" />;
  if (pathname === "/tags") return <Tags />;
  if (pathname === "/links") return <Links />;
  if (pathname === "/lint") return <LintPage />;
  if (pathname.startsWith("/documents/"))
    return <DocumentDetail id={decodeURIComponent(pathname.slice(11))} />;
  if (pathname.startsWith("/tag/"))
    return <TagDetailPage tag={decodeURIComponent(pathname.slice(5))} />;
  return (
    <div className="page not-found">
      <EmptyState>This page isn’t part of the document index.</EmptyState>
      <LinkButton to="/">Return to dashboard</LinkButton>
    </div>
  );
}

export function App() {
  const pathname = usePathname();
  return (
    <Layout pathname={pathname}>
      <CurrentPage pathname={pathname} />
    </Layout>
  );
}
