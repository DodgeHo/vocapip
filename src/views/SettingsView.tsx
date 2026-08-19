import { useEffect, useState } from "react";
import { AppShell } from "../components/AppShell";
import { Toggle } from "../components/Toggle";
import { zhCN } from "../i18n/zh-CN";
import { api } from "../lib/api";
import type {
  AppSettings,
  BootstrapPayload,
  LicenseDocument,
} from "../lib/types";
export function SettingsView() {
  const [data, setData] = useState<BootstrapPayload | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [licenses, setLicenses] = useState<LicenseDocument[] | null>(null);
  const [licenseOpen, setLicenseOpen] = useState(false);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  useEffect(() => {
    void api
      .bootstrap()
      .then((next) => {
        setData(next);
        setSettings(next.settings);
      })
      .catch(() => setError(zhCN.common.unknownError));
  }, []);
  const patch = (next: Partial<AppSettings>) =>
    setSettings((current) => (current ? { ...current, ...next } : current));
  const applyAlwaysOnTop = async (checked: boolean) => {
    const previous = settings?.alwaysOnTop ?? false;
    patch({ alwaysOnTop: checked });
    try {
      const applied = await api.setAlwaysOnTop(checked);
      patch({ alwaysOnTop: applied });
      setStatus(zhCN.settings.appliedImmediately);
      setError("");
    } catch {
      patch({ alwaysOnTop: previous });
      setStatus("");
      setError(zhCN.common.unknownError);
    }
  };
  const saveSettings = async () => {
    if (!settings) return;
    setStatus(zhCN.common.loading);
    try {
      const autostart = await api.setAutostart(settings.autostart);
      const alwaysOnTop = await api.setAlwaysOnTop(settings.alwaysOnTop);
      const saved = await api.updateSettings({
        ...settings,
        autostart,
        alwaysOnTop,
      });
      setSettings(saved);
      setStatus(zhCN.common.saved);
      setError("");
    } catch {
      setStatus("");
      setError(zhCN.common.unknownError);
    }
  };
  const showLicenses = async () => {
    setLicenseOpen(true);
    if (!licenses)
      try {
        setLicenses(await api.getLicenseDocuments());
      } catch {
        setError(zhCN.common.unknownError);
      }
  };
  const exportData = async () => {
    try {
      const result = await api.exportData();
      if (result) setStatus(zhCN.settings.exportDone);
    } catch {
      setError(zhCN.common.unknownError);
    }
  };
  const importData = async () => {
    try {
      const result = await api.importData();
      if (result) {
        setData(result);
        setSettings(result.settings);
        setStatus(zhCN.settings.importDone);
      }
    } catch {
      setError(zhCN.common.unknownError);
    }
  };
  if (!settings || !data)
    return (
      <AppShell active="settings">
        <div className="list-message">{error || zhCN.common.loading}</div>
      </AppShell>
    );
  const pack = data.languagePacks.find(
    (item) => item.id === settings.languagePackId,
  );
  return (
    <AppShell active="settings">
      <header className="page-header">
        <div>
          <p className="eyebrow">{zhCN.appName}</p>
          <h1>{zhCN.settings.title}</h1>
          <p>{zhCN.settings.subtitle}</p>
        </div>
        <button
          type="button"
          className="primary save-settings"
          onClick={() => void saveSettings()}
        >
          {status || zhCN.common.save}
        </button>
      </header>
      {error && <div className="error-banner">{error}</div>}
      <div className="settings-layout">
        <section className="settings-section">
          <h2>{zhCN.settings.learning}</h2>
          <div className="setting-row">
            <div>
              <label htmlFor="reminder">{zhCN.settings.reminderInterval}</label>
              <p>{zhCN.settings.reminderHelp}</p>
            </div>
            <div className="number-control">
              <input
                id="reminder"
                type="number"
                min="10"
                max="240"
                step="5"
                value={settings.reminderIntervalMinutes}
                onChange={(event) =>
                  patch({ reminderIntervalMinutes: Number(event.target.value) })
                }
              />
              <span>{zhCN.settings.minutes}</span>
            </div>
          </div>
          <div className="setting-row">
            <div>
              <label htmlFor="quota">{zhCN.settings.dailyQuota}</label>
            </div>
            <div className="number-control">
              <input
                id="quota"
                type="number"
                min="1"
                max="100"
                value={settings.dailyQuota}
                onChange={(event) =>
                  patch({ dailyQuota: Number(event.target.value) })
                }
              />
              <span>{zhCN.settings.words}</span>
            </div>
          </div>
          <div className="setting-row quiet-setting">
            <div>
              <label>{zhCN.settings.quietHours}</label>
              <p>{zhCN.settings.quietHelp}</p>
            </div>
            <Toggle
              label={zhCN.settings.quietHours}
              checked={settings.quietHoursEnabled}
              onChange={(checked) => patch({ quietHoursEnabled: checked })}
            />
            <div className="time-controls">
              <label>
                {zhCN.settings.start}
                <input
                  type="time"
                  value={settings.quietStart}
                  disabled={!settings.quietHoursEnabled}
                  onChange={(event) =>
                    patch({ quietStart: event.target.value })
                  }
                />
              </label>
              <span>→</span>
              <label>
                {zhCN.settings.end}
                <input
                  type="time"
                  value={settings.quietEnd}
                  disabled={!settings.quietHoursEnabled}
                  onChange={(event) => patch({ quietEnd: event.target.value })}
                />
              </label>
            </div>
          </div>
        </section>
        <section className="settings-section">
          <h2>{zhCN.settings.desktop}</h2>
          <div className="setting-row">
            <div>
              <label>{zhCN.settings.autostart}</label>
              <p>{zhCN.settings.autostartHelp}</p>
            </div>
            <Toggle
              label={zhCN.settings.autostart}
              checked={settings.autostart}
              onChange={(checked) => patch({ autostart: checked })}
            />
          </div>
          <div className="setting-row">
            <div>
              <label>{zhCN.settings.alwaysOnTop}</label>
              <p>{zhCN.settings.alwaysOnTopHelp}</p>
            </div>
            <Toggle
              label={zhCN.settings.alwaysOnTop}
              checked={settings.alwaysOnTop}
              onChange={(checked) => void applyAlwaysOnTop(checked)}
            />
          </div>
        </section>
        <section className="settings-section">
          <h2>{zhCN.settings.language}</h2>
          <div className="setting-row">
            <div>
              <label htmlFor="pack">{zhCN.settings.languagePack}</label>
              {pack && (
                <p>
                  {zhCN.settings.packRecords.replace(
                    "{count}",
                    String(pack.recordCount),
                  )}{" "}
                  · {pack.sourceLanguage} → {pack.targetLanguage}
                </p>
              )}
            </div>
            <select
              id="pack"
              value={settings.languagePackId}
              onChange={(event) =>
                patch({ languagePackId: event.target.value })
              }
            >
              {data.languagePacks.map((item) => (
                <option value={item.id} key={item.id}>
                  {item.name} · v{item.packVersion}
                </option>
              ))}
            </select>
          </div>
        </section>
        <section className="settings-section">
          <h2>{zhCN.settings.data}</h2>
          <div className="action-row">
            <div>
              <strong>{zhCN.settings.export}</strong>
              <p>{zhCN.settings.exportHelp}</p>
            </div>
            <button
              type="button"
              className="secondary"
              onClick={() => void exportData()}
            >
              {zhCN.settings.export}
            </button>
          </div>
          <div className="action-row">
            <div>
              <strong>{zhCN.settings.import}</strong>
              <p>{zhCN.settings.importHelp}</p>
            </div>
            <button
              type="button"
              className="secondary"
              onClick={() => void importData()}
            >
              {zhCN.settings.import}
            </button>
          </div>
          <div className="action-row">
            <div>
              <strong>{zhCN.settings.licenses}</strong>
              <p>{zhCN.settings.licensesHelp}</p>
            </div>
            <button
              type="button"
              className="secondary"
              onClick={() => void showLicenses()}
            >
              {zhCN.settings.showLicenses}
            </button>
          </div>
        </section>
        <section className="privacy-note">
          <strong>{zhCN.settings.privacyTitle}</strong>
          <p>{zhCN.settings.privacyBody}</p>
        </section>
      </div>
      {licenseOpen && (
        <div className="modal-backdrop">
          <section
            className="license-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="license-title"
          >
            <header>
              <h2 id="license-title">{zhCN.licenses.title}</h2>
              <button
                type="button"
                className="close-button"
                aria-label={zhCN.common.close}
                onClick={() => setLicenseOpen(false)}
              >
                ×
              </button>
            </header>
            <div className="license-content">
              {licenses ? (
                licenses.map((document) => (
                  <article key={document.id}>
                    <h3>{document.title}</h3>
                    <pre>{document.body}</pre>
                  </article>
                ))
              ) : (
                <p>{zhCN.common.loading}</p>
              )}
            </div>
          </section>
        </div>
      )}
    </AppShell>
  );
}
