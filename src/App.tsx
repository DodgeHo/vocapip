import "./App.css";
import { Component, type ErrorInfo, type ReactNode } from "react";
import { LibraryView } from "./views/LibraryView";
import { PetView } from "./views/PetView";
import { SettingsView } from "./views/SettingsView";
export type AppView = "pet" | "library" | "settings";

export function getView(search = window.location.search): AppView {
  const view = new URLSearchParams(search).get("view");
  return view === "library" || view === "settings" ? view : "pet";
}

interface AppErrorBoundaryState {
  hasError: boolean;
  message: string;
}

class AppErrorBoundary extends Component<
  { children: ReactNode },
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = { hasError: false, message: "" };

  static getDerivedStateFromError(error: Error): AppErrorBoundaryState {
    return { hasError: true, message: error.message || "页面渲染失败" };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[vocapip] renderer error", error, info.componentStack);
  }

  handleReload = () => window.location.reload();

  render() {
    if (!this.state.hasError) return this.props.children;
    return (
      <main className="fatal-error-screen" role="alert">
        <div className="fatal-error-panel">
          <span className="fatal-error-kicker">VocaPip</span>
          <h1>这个页面暂时没准备好</h1>
          <p>{this.state.message}</p>
          <button type="button" onClick={this.handleReload}>
            重新加载
          </button>
        </div>
      </main>
    );
  }
}

function AppViewRouter() {
  const view = getView();
  if (view === "library") return <LibraryView />;
  if (view === "settings") return <SettingsView />;
  return <PetView />;
}

function App() {
  return (
    <AppErrorBoundary>
      <AppViewRouter />
    </AppErrorBoundary>
  );
}

export default App;
