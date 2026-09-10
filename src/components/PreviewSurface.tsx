import { checkerboardStyle } from "./Checkerboard";
import { AppPhase } from "../App";
import { convertFileSrc } from "@tauri-apps/api/core";

interface Props {
  phase: AppPhase;
  previewPath: string | null;
  errorMessage: string | null;
}

export function PreviewSurface({ phase, previewPath, errorMessage }: Props) {
  return (
    <div
      style={{
        flex: 1,
        margin: 20,
        borderRadius: 12,
        overflow: "hidden",
        position: "relative",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        ...checkerboardStyle(),
      }}
    >
      {previewPath && (phase === "done" || phase === "processing") && (
        <img
          src={convertFileSrc(previewPath)}
          alt="Background removed preview"
          style={{
            maxWidth: "100%",
            maxHeight: "100%",
            objectFit: "contain",
            opacity: phase === "processing" ? 0.4 : 1,
            filter: phase === "processing" ? "blur(1px)" : "none",
            transition: "opacity 200ms ease, filter 200ms ease",
          }}
        />
      )}

      {(phase === "loading_model" || phase === "processing") && (
        <div
          style={{
            position: "absolute",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 12,
            background: "rgba(22, 22, 22, 0.55)",
            padding: "20px 28px",
            borderRadius: 10,
            backdropFilter: "blur(4px)",
          }}
        >
          <Spinner />
          <div style={{ fontSize: 13, color: "#e8e6e1" }}>
            {phase === "loading_model" ? "Loading AI model\u2026" : "Removing background\u2026"}
          </div>
        </div>
      )}

      {phase === "error" && errorMessage && (
        <div
          style={{
            position: "absolute",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 8,
            textAlign: "center",
            padding: "0 40px",
          }}
        >
          <div style={{ fontSize: 24 }}>&#9888;</div>
          <div style={{ fontSize: 14, color: "#e8746a", maxWidth: 360 }}>{errorMessage}</div>
        </div>
      )}
    </div>
  );
}

function Spinner() {
  return (
    <svg width="22" height="22" viewBox="0 0 22 22" style={{ animation: "spin 0.9s linear infinite" }}>
      <circle
        cx="11"
        cy="11"
        r="9"
        fill="none"
        stroke="#3a3a3a"
        strokeWidth="2.5"
      />
      <path
        d="M20 11A9 9 0 0 0 11 2"
        fill="none"
        stroke="#9fb4c4"
        strokeWidth="2.5"
        strokeLinecap="round"
      />
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </svg>
  );
}
