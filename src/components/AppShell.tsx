import { BookOpen, Settings } from "lucide-react";
import type { ReactNode } from "react";
import { zhCN } from "../i18n/zh-CN";
import { api } from "../lib/api";
interface AppShellProps {
  active: "library" | "settings";
  children: ReactNode;
}
export function AppShell({ active, children }: AppShellProps) {
  return (
    <div className="workspace-shell">
      <aside className="sidebar">
        <div className="brand-mark">
          <span className="brand-face">LP</span>
          <span>{zhCN.appName}</span>
        </div>
        <nav aria-label={zhCN.nav.ariaLabel}>
          <button
            type="button"
            className={active === "library" ? "active" : ""}
            onClick={() => void api.openWindow("library")}
          >
            <BookOpen aria-hidden="true" size={18} />
            {zhCN.nav.library}
          </button>
          <button
            type="button"
            className={active === "settings" ? "active" : ""}
            onClick={() => void api.openWindow("settings")}
          >
            <Settings aria-hidden="true" size={18} />
            {zhCN.nav.settings}
          </button>
        </nav>
        <div className="offline-badge">
          <span />
          {zhCN.common.offline}
        </div>
      </aside>
      <main className="workspace-main">{children}</main>
    </div>
  );
}
