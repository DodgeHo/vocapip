import { ImagePlus, RotateCcw, Trash2, Upload } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { AppShell } from "../components/AppShell";
import { PetAppearancePreview } from "../components/PetCharacter";
import { Toggle } from "../components/Toggle";
import { zhCN } from "../i18n/zh-CN";
import { api } from "../lib/api";
import type {
  AppSettings,
  BootstrapPayload,
  CustomPetAssets,
  LicenseDocument,
  PetAppearanceId,
  PetAssetState,
} from "../lib/types";

const MAX_CUSTOM_PET_BYTES = 5 * 1024 * 1024;
const MAX_CUSTOM_PET_DIMENSION = 4096;
const supportedPetImageTypes = new Set([
  "image/png",
  "image/jpeg",
  "image/webp",
]);
const petAppearances: PetAppearanceId[] = [
  "mint",
  "coral",
  "sun",
  "night",
  "custom",
];
const customPetSlots: PetAssetState[] = [
  "default",
  "idle",
  "click",
  "error",
  "correct",
  "upgrade",
];

function describeSettingsError(prefix: string, cause: unknown): string {
  const detail = cause instanceof Error ? cause.message.trim() : "";
  return detail ? `${prefix}：${detail}` : `${prefix}。`;
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () =>
      typeof reader.result === "string"
        ? resolve(reader.result)
        : reject(new Error(zhCN.settings.petImageReadError));
    reader.onerror = () => reject(new Error(zhCN.settings.petImageReadError));
    reader.readAsDataURL(file);
  });
}

function readImageDimensions(
  dataUrl: string,
): Promise<{ width: number; height: number }> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () =>
      resolve({ width: image.naturalWidth, height: image.naturalHeight });
    image.onerror = () => reject(new Error(zhCN.settings.petImageReadError));
    image.src = dataUrl;
  });
}

export async function validateCustomPetFile(file: File): Promise<string> {
  if (!supportedPetImageTypes.has(file.type))
    throw new Error(zhCN.settings.petImageTypeError);
  if (file.size > MAX_CUSTOM_PET_BYTES)
    throw new Error(zhCN.settings.petImageSizeError);
  const dataUrl = await readFileAsDataUrl(file);
  const { width, height } = await readImageDimensions(dataUrl);
  if (
    width < 1 ||
    height < 1 ||
    width > MAX_CUSTOM_PET_DIMENSION ||
    height > MAX_CUSTOM_PET_DIMENSION
  )
    throw new Error(zhCN.settings.petImageDimensionError);
  return dataUrl;
}

export function SettingsView() {
  const [data, setData] = useState<BootstrapPayload | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [licenses, setLicenses] = useState<LicenseDocument[] | null>(null);
  const [licenseOpen, setLicenseOpen] = useState(false);
  const [status, setStatus] = useState("");
  const [saving, setSaving] = useState(false);
  const [assetBusy, setAssetBusy] = useState(false);
  const [appearanceBusy, setAppearanceBusy] = useState(false);
  const [dataBusy, setDataBusy] = useState(false);
  const [error, setError] = useState("");
  const [customAssets, setCustomAssets] = useState<CustomPetAssets>({});
  const [petImageError, setPetImageError] = useState("");
  const mounted = useRef(false);
  const loadSequence = useRef(0);
  const modifying = saving || assetBusy || appearanceBusy || dataBusy;
  const load = async () => {
    const request = ++loadSequence.current;
    if (!mounted.current) return;
    setError("");
    try {
      const next = await api.bootstrap();
      const assets = await api.getPetAssets();
      if (!mounted.current || request !== loadSequence.current) return;
      setData(next);
      setSettings(next.settings);
      setCustomAssets(assets);
    } catch (cause) {
      if (mounted.current && request === loadSequence.current)
        setError(describeSettingsError(zhCN.settings.loadError, cause));
    }
  };
  useEffect(() => {
    mounted.current = true;
    void load();
    if (!("__TAURI_INTERNALS__" in window))
      return () => {
        mounted.current = false;
      };
    let cancelled = false;
    const disposers: Array<() => void> = [];
    void Promise.allSettled([
      listen("vocapip://settings-changed", () => void load()),
      listen("vocapip://pet-assets-changed", () => void load()),
      listen("vocapip://pet-appearance-changed", () => void load()),
      listen("vocapip://backup-imported", () => void load()),
    ]).then((results) => {
      const failed = results.find((result) => result.status === "rejected");
      results.forEach((result) => {
        if (result.status === "fulfilled") {
          if (cancelled) result.value();
          else disposers.push(result.value);
        }
      });
      if (failed && mounted.current)
        setError(
          describeSettingsError(zhCN.settings.eventSyncError, failed.reason),
        );
    });
    return () => {
      cancelled = true;
      mounted.current = false;
      disposers.forEach((dispose) => dispose());
    };
  }, []);
  const patch = (next: Partial<AppSettings>) => {
    if (!mounted.current) return;
    setSettings((current) => (current ? { ...current, ...next } : current));
  };
  const saveSettings = async () => {
    if (!settings || modifying) return;
    setSaving(true);
    setStatus(zhCN.common.loading);
    try {
      const saved = await api.updateSettings(settings);
      if (!mounted.current) return;
      setData((current) =>
        current ? { ...current, settings: saved } : current,
      );
      setSettings(saved);
      setStatus(zhCN.common.saved);
      setError("");
    } catch (cause) {
      if (mounted.current) {
        setStatus("");
        setError(describeSettingsError(zhCN.settings.saveError, cause));
      }
    } finally {
      if (mounted.current) setSaving(false);
    }
  };
  const showLicenses = async () => {
    setLicenseOpen(true);
    if (!licenses)
      try {
        const next = await api.getLicenseDocuments();
        if (mounted.current) setLicenses(next);
      } catch (cause) {
        if (mounted.current)
          setError(
            describeSettingsError(zhCN.settings.licenseLoadError, cause),
          );
      }
  };
  const exportData = async () => {
    if (modifying) return;
    try {
      const result = await api.exportData();
      if (result && mounted.current) setStatus(zhCN.settings.exportDone);
    } catch (cause) {
      if (mounted.current)
        setError(describeSettingsError(zhCN.settings.exportError, cause));
    }
  };
  const importData = async () => {
    if (modifying) return;
    setDataBusy(true);
    try {
      const result = await api.importData();
      if (result) {
        const refreshed = await api.bootstrap();
        const assets = await api.getPetAssets();
        if (!mounted.current) return;
        setData(refreshed);
        setSettings(refreshed.settings);
        setCustomAssets(assets);
        setStatus(zhCN.settings.importDone);
      }
    } catch (cause) {
      if (mounted.current)
        setError(describeSettingsError(zhCN.settings.importError, cause));
    } finally {
      if (mounted.current) setDataBusy(false);
    }
  };
  const choosePetAppearance = async (
    appearanceId: PetAppearanceId,
  ): Promise<boolean> => {
    if (appearanceBusy) return false;
    const previous = settings?.petAppearanceId ?? "mint";
    setAppearanceBusy(true);
    patch({ petAppearanceId: appearanceId });
    try {
      const applied = await api.setPetAppearance(appearanceId);
      if (!mounted.current) return false;
      patch({ petAppearanceId: applied });
      setStatus(zhCN.settings.appliedImmediately);
      setError("");
      return true;
    } catch (cause) {
      if (mounted.current) {
        patch({ petAppearanceId: previous });
        setError(
          describeSettingsError(zhCN.settings.petAppearanceError, cause),
        );
      }
      return false;
    } finally {
      if (mounted.current) setAppearanceBusy(false);
    }
  };
  const uploadCustomPet = async (
    state: PetAssetState,
    file: File | undefined,
  ) => {
    if (!file || modifying) return;
    setAssetBusy(true);
    setPetImageError("");
    try {
      const dataUrl = await validateCustomPetFile(file);
      const saved = await api.importPetImage(state, dataUrl);
      if (!mounted.current) return;
      setCustomAssets((current) => ({
        ...current,
        [state]: saved.dataUrl,
      }));
      const canEnableCustom =
        state === "default" || Boolean(customAssets.default);
      if (canEnableCustom) {
        const enabled =
          state === "default" ? true : await choosePetAppearance("custom");
        if (state === "default") patch({ petAppearanceId: "custom" });
        if (enabled && mounted.current) setStatus(zhCN.settings.petImageReady);
      } else {
        setStatus(zhCN.settings.petImageSavedNeedsDefault);
      }
    } catch (cause) {
      if (mounted.current)
        setPetImageError(
          describeSettingsError(zhCN.settings.customPetSaveError, cause),
        );
    } finally {
      if (mounted.current) setAssetBusy(false);
    }
  };
  const removeCustomPetImage = async (state: PetAssetState) => {
    if (modifying) return;
    setAssetBusy(true);
    try {
      const appearance = await api.removePetImage(state);
      if (!mounted.current) return;
      setCustomAssets((current) => {
        const next = { ...current };
        delete next[state];
        return next;
      });
      patch({ petAppearanceId: appearance });
      setPetImageError("");
      setStatus(zhCN.settings.petImageRemoved);
    } catch (cause) {
      if (mounted.current)
        setPetImageError(
          describeSettingsError(zhCN.settings.customPetRemoveError, cause),
        );
    } finally {
      if (mounted.current) setAssetBusy(false);
    }
  };
  const clearCustomPet = async () => {
    if (modifying) return;
    setAssetBusy(true);
    try {
      const appearance = await api.clearPetImages();
      if (!mounted.current) return;
      setCustomAssets({});
      patch({ petAppearanceId: appearance });
      setPetImageError("");
      setStatus(zhCN.settings.petImagesRemoved);
    } catch (cause) {
      if (mounted.current)
        setPetImageError(
          describeSettingsError(zhCN.settings.customPetClearError, cause),
        );
    } finally {
      if (mounted.current) setAssetBusy(false);
    }
  };
  if (!settings || !data)
    return (
      <AppShell active="settings">
        <div className="list-message">
          <strong>{error || zhCN.common.loading}</strong>
          {error && (
            <button
              type="button"
              className="secondary"
              onClick={() => void load()}
            >
              {zhCN.common.retry}
            </button>
          )}
        </div>
      </AppShell>
    );
  const pack = data.languagePacks.find(
    (item) => item.id === settings.languagePackId,
  );
  const hasCustomAssets = customPetSlots.some((state) => customAssets[state]);
  const hasCustomDefaultAsset = Boolean(customAssets.default);
  return (
    <AppShell active="settings">
      <header className="page-header">
        <div>
          <p className="eyebrow">{zhCN.appName}</p>
          <h1>{zhCN.settings.title}</h1>
          <p>{zhCN.settings.subtitle}</p>
        </div>
        <div className="settings-save-actions">
          <button
            type="button"
            className="primary save-settings"
            disabled={modifying}
            onClick={() => void saveSettings()}
          >
            {saving ? zhCN.common.loading : zhCN.common.save}
          </button>
          {status && <span role="status">{status}</span>}
        </div>
      </header>
      {error && (
        <div className="error-banner" role="alert">
          <span>{error}</span>
          <button type="button" onClick={() => void load()}>
            {zhCN.common.retry}
          </button>
        </div>
      )}
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
                disabled={modifying}
                onChange={(event) =>
                  patch({
                    reminderIntervalMinutes: Math.min(
                      240,
                      Math.max(10, Number(event.target.value) || 10),
                    ),
                  })
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
                disabled={modifying}
                onChange={(event) =>
                  patch({
                    dailyQuota: Math.min(
                      100,
                      Math.max(1, Number(event.target.value) || 1),
                    ),
                  })
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
              disabled={modifying}
              onChange={(checked) => patch({ quietHoursEnabled: checked })}
            />
            <div className="time-controls">
              <label>
                {zhCN.settings.start}
                <input
                  type="time"
                  value={settings.quietStart}
                  disabled={!settings.quietHoursEnabled || modifying}
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
                  disabled={!settings.quietHoursEnabled || modifying}
                  onChange={(event) => patch({ quietEnd: event.target.value })}
                />
              </label>
            </div>
          </div>
        </section>
        <section className="settings-section">
          <h2>{zhCN.settings.speech}</h2>
          <div className="setting-row">
            <div>
              <label>{zhCN.settings.speechAutoRead}</label>
              <p>{zhCN.settings.speechHelp}</p>
            </div>
            <Toggle
              label={zhCN.settings.speechAutoRead}
              checked={settings.speechEnabled}
              disabled={modifying}
              onChange={(checked) => patch({ speechEnabled: checked })}
            />
          </div>
        </section>
        <section className="settings-section pet-appearance-section">
          <h2>{zhCN.settings.petAppearance}</h2>
          <p className="section-help">{zhCN.settings.petAppearanceHelp}</p>
          <div
            className="appearance-picker"
            role="radiogroup"
            aria-label={zhCN.settings.petAppearance}
          >
            {petAppearances.map((appearance) => (
              <button
                type="button"
                role="radio"
                aria-checked={settings.petAppearanceId === appearance}
                disabled={
                  modifying ||
                  (appearance === "custom" && !hasCustomDefaultAsset)
                }
                className={
                  "appearance-option" +
                  (settings.petAppearanceId === appearance
                    ? " is-selected"
                    : "")
                }
                onClick={() => void choosePetAppearance(appearance)}
                key={appearance}
              >
                {appearance === "custom" && !hasCustomDefaultAsset ? (
                  <span
                    className="appearance-upload-placeholder"
                    aria-hidden="true"
                  >
                    <ImagePlus size={27} />
                  </span>
                ) : (
                  <PetAppearancePreview
                    appearanceId={appearance}
                    customAssets={customAssets}
                  />
                )}
                <span>{zhCN.settings.petAppearances[appearance]}</span>
              </button>
            ))}
          </div>
          <div className="custom-pet-heading">
            <strong>{zhCN.settings.customPetImage}</strong>
            <p>{zhCN.settings.customPetImageHelp}</p>
          </div>
          <div className="custom-pet-slots">
            {customPetSlots.map((state) => {
              const uploadLabel = zhCN.settings.uploadPetImageFor.replace(
                "{state}",
                zhCN.settings.customPetSlots[state],
              );
              const asset = customAssets[state];
              return (
                <div className="custom-pet-slot" key={state}>
                  <div className="custom-pet-slot-preview">
                    {asset ? (
                      <PetAppearancePreview
                        appearanceId="custom"
                        customAssets={{ [state]: asset }}
                        mood={state === "default" ? "idle" : state}
                      />
                    ) : (
                      <ImagePlus aria-hidden="true" size={24} />
                    )}
                  </div>
                  <div className="custom-pet-slot-copy">
                    <strong>{zhCN.settings.customPetSlots[state]}</strong>
                    <span>{zhCN.settings.customPetSlotHelp[state]}</span>
                  </div>
                  <div className="custom-pet-slot-actions">
                    <label
                      className="icon-button"
                      htmlFor={`custom-pet-image-${state}`}
                      title={uploadLabel}
                      aria-disabled={modifying}
                    >
                      <Upload aria-hidden="true" size={16} />
                      <span className="visually-hidden">{uploadLabel}</span>
                    </label>
                    <input
                      id={`custom-pet-image-${state}`}
                      className="visually-hidden"
                      type="file"
                      accept="image/png,image/jpeg,image/webp"
                      aria-label={uploadLabel}
                      disabled={modifying}
                      onChange={(event) => {
                        void uploadCustomPet(state, event.target.files?.[0]);
                        event.currentTarget.value = "";
                      }}
                    />
                    {asset && (
                      <button
                        type="button"
                        className="icon-button"
                        title={`${zhCN.settings.removePetImage}：${zhCN.settings.customPetSlots[state]}`}
                        aria-label={`${zhCN.settings.removePetImage}：${zhCN.settings.customPetSlots[state]}`}
                        disabled={modifying}
                        onClick={() => void removeCustomPetImage(state)}
                      >
                        <Trash2 aria-hidden="true" size={16} />
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
          <div className="custom-pet-footer-actions">
            <button
              type="button"
              className="secondary"
              disabled={modifying}
              onClick={() => void choosePetAppearance("mint")}
            >
              <RotateCcw aria-hidden="true" size={16} />
              {zhCN.settings.restoreSystemPet}
            </button>
            {hasCustomAssets && (
              <button
                type="button"
                className="secondary danger-action"
                disabled={modifying}
                onClick={() => void clearCustomPet()}
              >
                <Trash2 aria-hidden="true" size={16} />
                {zhCN.settings.removeAllPetImages}
              </button>
            )}
          </div>
          {petImageError && (
            <p className="inline-error pet-image-error">{petImageError}</p>
          )}
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
              disabled={modifying}
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
              disabled={modifying}
              onChange={(checked) => patch({ alwaysOnTop: checked })}
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
                  {pack.composite ? "全部词库" : pack.name}·{" "}
                  {pack.recordCount.toLocaleString("zh-CN")} 条记录 ·{" "}
                  {pack.uniqueWordCount.toLocaleString("zh-CN")} 个独立单词 ·{" "}
                  {pack.sourceLanguage} → {pack.targetLanguage}
                </p>
              )}
            </div>
            <select
              id="pack"
              value={settings.languagePackId}
              disabled={modifying}
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
              disabled={modifying}
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
              disabled={modifying}
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
