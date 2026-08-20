import { Search, Star, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { AppShell } from "../components/AppShell";
import { zhCN } from "../i18n/zh-CN";
import { api } from "../lib/api";
import type { LearningCard, LibraryPayload, LibraryQuery } from "../lib/types";

const filters: LibraryQuery["status"][] = [
  "all",
  "new",
  "learning",
  "review",
  "relearning",
  "favorite",
  "recent",
];
const PAGE_SIZE = 60;

function formatDate(value?: string): string {
  if (!value) return "—";
  return new Intl.DateTimeFormat("zh-CN", {
    month: "short",
    day: "numeric",
  }).format(new Date(value));
}

export function LibraryView() {
  const [query, setQuery] = useState<LibraryQuery>({
    search: "",
    status: "all",
    limit: PAGE_SIZE,
    offset: 0,
  });
  const [payload, setPayload] = useState<LibraryPayload | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [notice, setNotice] = useState("");
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const next = await api.listLibrary({ ...query, offset: 0 });
      setPayload(next);
      setError("");
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : zhCN.common.unknownError,
      );
    } finally {
      setLoading(false);
    }
  }, [query]);

  useEffect(() => {
    const timer = window.setTimeout(() => void load(), 180);
    return () => window.clearTimeout(timer);
  }, [load]);

  const loadMore = async () => {
    if (!payload?.hasMore || loadingMore) return;
    setLoadingMore(true);
    try {
      const next = await api.listLibrary({
        ...query,
        offset: payload.items.length,
      });
      setPayload((current) =>
        current
          ? {
              ...next,
              items: [...current.items, ...next.items],
            }
          : next,
      );
      setError("");
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : zhCN.common.unknownError,
      );
    } finally {
      setLoadingMore(false);
    }
  };

  const replace = (updated: LearningCard) =>
    setPayload((current) =>
      current
        ? {
            ...current,
            items: current.items.map((item) =>
              item.id === updated.id ? updated : item,
            ),
          }
        : current,
    );

  const toggleFavorite = async (card: LearningCard) => {
    try {
      replace(await api.toggleFavorite(card.id));
    } catch {
      setError(zhCN.common.unknownError);
    }
  };

  const relearn = async (card: LearningCard) => {
    try {
      replace(await api.queueRelearn(card.id));
      setNotice(card.word + " · " + zhCN.library.queued);
      window.setTimeout(() => setNotice(""), 1800);
    } catch {
      setError(zhCN.common.unknownError);
    }
  };

  const updateQuery = (next: Partial<LibraryQuery>) =>
    setQuery((current) => ({ ...current, ...next }));

  return (
    <AppShell active="library">
      <header className="page-header">
        <div>
          <p className="eyebrow">{zhCN.appName}</p>
          <h1>{zhCN.library.title}</h1>
          <p>{zhCN.library.subtitle}</p>
        </div>
        <div className="header-stat">
          <strong>{payload?.counts.all ?? 0}</strong>
          <span>{zhCN.library.records}</span>
        </div>
      </header>

      <div className="library-toolbar">
        <label className="search-box">
          <Search aria-hidden="true" size={18} />
          <input
            value={query.search}
            onChange={(event) => updateQuery({ search: event.target.value })}
            placeholder={zhCN.library.searchPlaceholder}
            aria-label={zhCN.library.searchPlaceholder}
          />
          {query.search && (
            <button
              type="button"
              className="icon-button search-clear"
              onClick={() => updateQuery({ search: "" })}
              aria-label={zhCN.common.clear}
              title={zhCN.common.clear}
            >
              <X aria-hidden="true" size={16} />
            </button>
          )}
        </label>
        <div
          className="filter-tabs"
          role="tablist"
          aria-label={zhCN.library.title}
        >
          {filters.map((filter) => (
            <button
              type="button"
              role="tab"
              aria-selected={query.status === filter}
              key={filter}
              className={query.status === filter ? "active" : ""}
              onClick={() => updateQuery({ status: filter })}
            >
              {zhCN.library.filters[filter]}
              <span>{payload?.counts[filter] ?? 0}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="library-result-summary">
        <span>{zhCN.library.filters[query.status]}</span>
        <strong>{payload?.items.length ?? 0}</strong>
        {query.search && <span> · {query.search}</span>}
      </div>

      {notice && (
        <div className="toast" role="status">
          {notice}
        </div>
      )}
      {error && (
        <div className="error-banner" role="alert">
          <span>{error}</span>
          <button type="button" onClick={() => void load()}>
            {zhCN.common.retry}
          </button>
        </div>
      )}

      <section className="library-list" aria-busy={loading || loadingMore}>
        {loading && !payload ? (
          <div className="list-message">{zhCN.common.loading}</div>
        ) : payload?.items.length ? (
          <>
            {payload.items.map((card) => (
              <article className="word-row" key={card.id}>
                <div className="word-primary">
                  <button
                    type="button"
                    className={
                      "favorite-button " + (card.favorite ? "favorite" : "")
                    }
                    aria-label={
                      card.favorite
                        ? zhCN.library.unfavorite
                        : zhCN.library.favorite
                    }
                    title={
                      card.favorite
                        ? zhCN.library.unfavorite
                        : zhCN.library.favorite
                    }
                    onClick={() => void toggleFavorite(card)}
                  >
                    <Star
                      aria-hidden="true"
                      size={18}
                      fill={card.favorite ? "currentColor" : "none"}
                    />
                  </button>
                  <div>
                    <div className="word-title">
                      <h2>{card.word}</h2>
                      <span>{card.partOfSpeech}</span>
                      <span className="tier-chip">{card.tier}</span>
                    </div>
                    <p>{card.definitionZh}</p>
                  </div>
                </div>
                <div className="word-example">
                  <p>{card.exampleEn || zhCN.library.noExample}</p>
                  <span>{card.exampleZh || zhCN.library.noExample}</span>
                </div>
                <div className="word-meta">
                  <span className={"status status-" + card.status}>
                    {zhCN.library.status[card.status]}
                  </span>
                  {card.sourcePackName && (
                    <small className="source-pack" title={card.sourcePackName}>
                      {zhCN.library.source} · {card.sourcePackName}
                    </small>
                  )}
                  <small>
                    {card.nextReviewAt
                      ? zhCN.library.due + " " + formatDate(card.nextReviewAt)
                      : zhCN.library.lastStudied +
                        " " +
                        formatDate(card.lastStudiedAt)}
                  </small>
                </div>
                <button
                  type="button"
                  className="relearn-button"
                  onClick={() => void relearn(card)}
                >
                  {zhCN.library.relearn}
                </button>
              </article>
            ))}
            {payload.hasMore && (
              <button
                type="button"
                className="load-more-button"
                disabled={loadingMore}
                onClick={() => void loadMore()}
              >
                {loadingMore ? zhCN.common.loading : zhCN.library.loadMore}
              </button>
            )}
          </>
        ) : (
          <div className="list-message">
            <strong>{zhCN.library.emptyTitle}</strong>
            <p>{zhCN.library.emptyBody}</p>
          </div>
        )}
      </section>
    </AppShell>
  );
}
