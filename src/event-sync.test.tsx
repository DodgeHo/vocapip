// @vitest-environment jsdom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const eventState = vi.hoisted(() => ({
  listeners: new Map<string, () => void>(),
  disposers: new Map<string, ReturnType<typeof vi.fn>>(),
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: eventState.listen,
}));

import { api } from "./lib/api";
import { SettingsView } from "./views/SettingsView";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  eventState.listeners.clear();
  eventState.disposers.clear();
});

beforeEach(() => {
  eventState.listen.mockImplementation(
    (event: string, callback: () => void) => {
      eventState.listeners.set(event, callback);
      const dispose = vi.fn(() => eventState.listeners.delete(event));
      eventState.disposers.set(event, dispose);
      return Promise.resolve(dispose);
    },
  );
});

describe("Tauri window synchronization", () => {
  it("refreshes settings after a cross-window event and removes listeners on unmount", async () => {
    const initial = await api.bootstrap();
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    const bootstrap = vi.spyOn(api, "bootstrap").mockResolvedValue(initial);
    vi.spyOn(api, "getPetAssets").mockResolvedValue({});

    const { unmount } = render(<SettingsView />);
    await screen.findByRole("heading", { name: "学习节奏" });
    await waitFor(() =>
      expect(eventState.listeners.has("vocapip://settings-changed")).toBe(true),
    );

    eventState.listeners.get("vocapip://settings-changed")?.();
    await waitFor(() => expect(bootstrap).toHaveBeenCalledTimes(2));

    unmount();
    await waitFor(() => {
      for (const dispose of eventState.disposers.values())
        expect(dispose).toHaveBeenCalledTimes(1);
    });
  });
});
