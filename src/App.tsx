import "./App.css";
import { LibraryView } from "./views/LibraryView";
import { PetView } from "./views/PetView";
import { SettingsView } from "./views/SettingsView";
export type AppView = "pet" | "library" | "settings";
export function getView(search = window.location.search): AppView {
  const view = new URLSearchParams(search).get("view");
  return view === "library" || view === "settings" ? view : "pet";
}
function App() {
  const view = getView();
  if (view === "library") return <LibraryView />;
  if (view === "settings") return <SettingsView />;
  return <PetView />;
}
export default App;
