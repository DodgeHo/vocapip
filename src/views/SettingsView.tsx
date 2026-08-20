import { ImagePlus, RotateCcw, Trash2, Upload } from "lucide-react";
import { useEffect, useState } from "react";
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
  const [error, setError] = useState("");
  const [customAssets, setCustomAssets] = useState<CustomPetAssets>({});
  const [petImageError, setPetImageError] = useState("");
  const load = async () => {
    setError("");
    try {
      const [next, assets] = await Promise.all([
        api.bootstrap(),
        api.getPetAssets(),
      ]);
      setData(next);
      setSettings(next.settings);
      setCustomAssets(assets);
    } catch {
      setError(zhCN.common.unknownError);
    }
  };
  useEffect(() => {
    void load();
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
    if (!settings || saving) return;
    setSaving(true);
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
    } finally {
      setSaving(false);
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
  const choosePetAppearance = async (appearanceId: PetAppearanceId) => {
    const previous = settings?.petAppearanceId ?? "mint";
    patch({ petAppearanceId: appearanceId });
    try {
      const applied = await api.setPetAppearance(appearanceId);
      patch({ petAppearanceId: applied });
      setStatus(zhCN.settings.appliedImmediately);
      setError("");
    } catch {
      patch({ petAppearanceId: previous });
      setError(zhCN.common.unknownError);
    }
  };
  const uploadCustomPet = async (
    state: PetAssetState,
    file: File | undefined,
  ) => {
    if (!file) return;
    setPetImageError("");
    try {
      const dataUrl = await validateCustomPetFile(file);
      const saved = await api.importPetImage(state, dataUrl);
      setCustomAssets((current) => ({
        ...current,
        [state]: saved.dataUrl,
      }));
      await choosePetAppearance("custom");
      setStatus(zhCN.settings.petImageReady);
    } catch (cause) {
      setPetImageError(
        cause instanceof Error
          ? cause.message
          : zhCN.settings.petImageReadError,
      );
    }
  };
  const removeCustomPetImage = async (state: PetAssetState) => {
    try {
      await api.removePetImage(state);
      const remainingAssets = Object.entries(customAssets).filter(
        ([assetState, dataUrl]) => assetState !== state && Boolean(dataUrl),
      );
      setCustomAssets((current) => {
        const next = { ...current };
        delete next[state];
        return next;
      });
      if (settings?.petAppearanceId === "custom" && !remainingAssets.length)
        await choosePetAppearance("mint");
      setPetImageError("");
      setStatus(zhCN.settings.petImageRemoved);
    } catch {
      setPetImageError(zhCN.common.unknownError);
    }
  };
  const clearCustomPet = async () => {
    try {
      await Promise.all(
        customPetSlots.map((state) => api.removePetImage(state)),
      );
      setCustomAssets({});
      await choosePetAppearance("mint");
      setPetImageError("");
      setStatus(zhCN.settings.petImagesRemoved);
    } catch {
      setPetImageError(zhCN.common.unknownError);
    }
  };
  if (!settings || !data)
    return (
      <AppShell active="settings">
        <div className="list-message">
          <strong>
            {error ? zhCN.common.unknownError : zhCN.common.loading}
          </strong>
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
          <h2>{zhCN.settings.speech}</h2>
          <div className="setting-row">
            <div>
              <label>{zhCN.settings.speechAutoRead}</label>
              <p>{zhCN.settings.speechHelp}</p>
            </div>
            <Toggle
              label={zhCN.settings.speechAutoRead}
              checked={settings.speechEnabled}
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
                disabled={appearance === "custom" && !hasCustomAssets}
                className={
                  "appearance-option" +
                  (settings.petAppearanceId === appearance
                    ? " is-selected"
                    : "")
                }
                onClick={() => void choosePetAppearance(appearance)}
                key={appearance}
              >
                {appearance === "custom" && !hasCustomAssets ? (
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
              onClick={() => void choosePetAppearance("mint")}
            >
              <RotateCcw aria-hidden="true" size={16} />
              {zhCN.settings.restoreSystemPet}
            </button>
            {hasCustomAssets && (
              <button
                type="button"
                className="secondary danger-action"
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
