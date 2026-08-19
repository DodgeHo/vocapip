import { useState } from "react";
import { zhCN } from "../i18n/zh-CN";
import type {
  LevelBand,
  PlacementAnswer,
  PlacementSubmission,
} from "../lib/types";
const tiers: LevelBand[] = ["T1", "T2", "T3", "T4", "T5"];
const questions = [
  ["q1", "daily", "T1"],
  ["q2", "choose", "T1"],
  ["q3", "notice", "T2"],
  ["q4", "improve", "T2"],
  ["q5", "reliable", "T3"],
  ["q6", "approach", "T3"],
  ["q7", "subtle", "T4"],
  ["q8", "allocate", "T4"],
  ["q9", "coherent", "T5"],
  ["q10", "ubiquitous", "T5"],
  ["q11", "mitigate", "T5"],
  ["q12", "contingent", "T5"],
] as const;
function scoreTier(answers: PlacementAnswer[]): LevelBand {
  const known = answers.filter((answer) => answer.correct).length;
  if (known <= 2) return "T1";
  if (known <= 4) return "T2";
  if (known <= 7) return "T3";
  if (known <= 10) return "T4";
  return "T5";
}
export function Placement({
  onComplete,
}: {
  onComplete: (submission: PlacementSubmission) => Promise<void>;
}) {
  const [mode, setMode] = useState<"self" | "quiz">("self");
  const [tier, setTier] = useState<LevelBand>("T2");
  const [questionIndex, setQuestionIndex] = useState(0);
  const [answers, setAnswers] = useState<PlacementAnswer[]>([]);
  const [busy, setBusy] = useState(false);
  const answer = (correct: boolean) => {
    const next = [
      ...answers,
      { questionId: questions[questionIndex][0], correct },
    ];
    setAnswers(next);
    if (questionIndex < questions.length - 1)
      setQuestionIndex(questionIndex + 1);
    else setTier(scoreTier(next));
  };
  const quizDone = answers.length === questions.length;
  const submit = async () => {
    setBusy(true);
    try {
      await onComplete({
        selfReportedTier: tier,
        answers: mode === "quiz" ? answers : undefined,
      });
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="placement-backdrop">
      <section className="placement-panel" aria-labelledby="placement-title">
        <p className="eyebrow">{zhCN.placement.eyebrow}</p>
        <h1 id="placement-title">{zhCN.placement.title}</h1>
        <p className="muted">{zhCN.placement.body}</p>
        <div className="segmented" role="tablist">
          <button
            type="button"
            className={mode === "self" ? "active" : ""}
            onClick={() => setMode("self")}
          >
            {zhCN.placement.selfReport}
          </button>
          <button
            type="button"
            className={mode === "quiz" ? "active" : ""}
            onClick={() => setMode("quiz")}
          >
            {zhCN.placement.quiz}
          </button>
        </div>
        {mode === "self" ? (
          <div className="tier-list">
            {tiers.map((item) => (
              <button
                type="button"
                className={tier === item ? "selected" : ""}
                onClick={() => setTier(item)}
                key={item}
              >
                <strong>{zhCN.placement.tiers[item].title}</strong>
                <span>{zhCN.placement.tiers[item].detail}</span>
                <i aria-hidden="true" />
              </button>
            ))}
          </div>
        ) : (
          <div className="quiz-panel">
            {!quizDone ? (
              <>
                <div className="quiz-progress">
                  <span>
                    {zhCN.placement.questionProgress} {questionIndex + 1} /{" "}
                    {questions.length}
                  </span>
                  <div>
                    <i
                      style={{
                        width:
                          ((questionIndex + 1) / questions.length) * 100 + "%",
                      }}
                    />
                  </div>
                </div>
                <div className="quiz-word">
                  <span>{questions[questionIndex][2]}</span>
                  <strong>{questions[questionIndex][1]}</strong>
                </div>
                <div className="quiz-actions">
                  <button
                    type="button"
                    className="secondary"
                    onClick={() => answer(false)}
                  >
                    {zhCN.placement.doNotKnow}
                  </button>
                  <button
                    type="button"
                    className="primary"
                    onClick={() => answer(true)}
                  >
                    {zhCN.placement.know}
                  </button>
                </div>
              </>
            ) : (
              <div className="quiz-result">
                <span className="result-tier">{tier}</span>
                <h2>{zhCN.placement.result.replace("{tier}", tier)}</h2>
                <p>{zhCN.placement.tiers[tier].detail}</p>
              </div>
            )}
          </div>
        )}
        <button
          type="button"
          className="primary placement-submit"
          disabled={busy || (mode === "quiz" && !quizDone)}
          onClick={() => void submit()}
        >
          {busy ? zhCN.common.loading : zhCN.placement.submit}
        </button>
      </section>
    </div>
  );
}
