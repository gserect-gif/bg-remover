import { AppPhase } from "../App";

interface Props {
  phase: AppPhase;
  fileName: string | null;
  width: number | null;
  height: number | null;
  onExport: () => void;
  onReset: () => void;
  onBrowse: () => void;
}

export function ControlRail({ phase, fileName, width, height, onExport, onReset, onBrowse }: Props) {
  return (
    <div
      style={{
        width: 260,
        flexShrink: 0,
        borderLeft: "1px solid #322f2c",
        background: "#1c1c1c",
        display: "flex",
        flexDirection: "column",
        padding: "24px 20px",
        gap: 24,
      }}
    >
      <div>
        <div style={{ fontSize: 11, fontWeight: 600, letterSpacing: 0.2, color: "#8b8781", marginBottom: 10 }}>
          Status
        </div>
        <StatusRow phase={phase} />
      </div>

      {fileName && (
        <div>
          <div style={{ fontSize: 11, fontWeight: 600, letterSpacing: 0.2, color: "#8b8781", marginBottom: 10 }}>
            File
          </div>
          <div
            style={{
              fontSize: 13,
              color: "#e8e6e1",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
            title={fileName}
          >
            {fileName}
          </div>
          {width && height && (
            <div className="tabular" style={{ fontSize: 12, color: "#8b8781", marginTop: 4 }}>
              {width} &times; {height}px
            </div>
          )}
        </div>
      )}

      <div style={{ flex: 1 }} />

      <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
        <button
          onClick={onExport}
          disabled={phase !== "done"}
          style={{
            padding: "11px 16px",
            borderRadius: 8,
            border: "none",
            background: phase === "done" ? "#7a8b99" : "#2a2a2a",
            color: phase === "done" ? "#161616" : "#5a5652",
            fontSize: 13,
            fontWeight: 600,
            cursor: phase === "done" ? "pointer" : "default",
            transition: "background 120ms ease, transform 80ms ease",
          }}
          onMouseDown={(e) => {
            if (phase === "done") e.currentTarget.style.transform = "scale(0.98)";
          }}
          onMouseUp={(e) => {
            e.currentTarget.style.transform = "scale(1)";
          }}
        >
          Export transparent PNG
        </button>

        {fileName && (
          <button
            onClick={onReset}
            style={{
              padding: "10px 16px",
              borderRadius: 8,
              border: "1px solid #322f2c",
              background: "transparent",
              color: "#8b8781",
              fontSize: 13,
              fontWeight: 500,
              cursor: "pointer",
            }}
          >
            {phase === "done" ? "Start over" : "Cancel"}
          </button>
        )}

        {!fileName && (
          <button
            onClick={onBrowse}
            style={{
              padding: "10px 16px",
              borderRadius: 8,
              border: "1px solid #322f2c",
              background: "transparent",
              color: "#8b8781",
              fontSize: 13,
              fontWeight: 500,
              cursor: "pointer",
            }}
          >
            Browse for image&hellip;
          </button>
        )}
      </div>
    </div>
  );
}

function StatusRow({ phase }: { phase: AppPhase }) {
  const map: Record<AppPhase, { label: string; color: string; pulse: boolean }> = {
    idle: { label: "Waiting for image", color: "#5a5652", pulse: false },
    loading_model: { label: "Loading AI model\u2026", color: "#9fb4c4", pulse: true },
    processing: { label: "Removing background\u2026", color: "#9fb4c4", pulse: true },
    done: { label: "Complete", color: "#5fd3a0", pulse: false },
    error: { label: "Failed", color: "#e8746a", pulse: false },
  };
  const { label, color, pulse } = map[phase];

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
      <span
        style={{
          width: 7,
          height: 7,
          borderRadius: "50%",
          background: color,
          animation: pulse ? "pulse 1.4s ease-in-out infinite" : "none",
        }}
      />
      <span style={{ fontSize: 13, color: "#e8e6e1" }}>{label}</span>
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.35; }
        }
      `}</style>
    </div>
  );
}
