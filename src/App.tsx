import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

interface WpmPayload {
  current_wpm: number;
  raw_wpm: number;
}

function App() {
  const [wpm, setWpm] = useState(0);
  const [rawWpm, setRawWpm] = useState(0);

  useEffect(() => {
    // Listen for WPM updates from the Rust backend
    const unlisten = listen<WpmPayload>("wpm-update", (event) => {
      setWpm(event.payload.current_wpm);
      setRawWpm(event.payload.raw_wpm);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const handleMouseDown = (e: React.MouseEvent) => {
    // Prevent dragging if clicking on text to allow selection if needed
    // though for a widget, grab anywhere is fine.
    getCurrentWindow().startDragging();
  };

  return (
    <main className="widget-container" onMouseDown={handleMouseDown} style={{ cursor: "grab" }}>
      <div className="wpm-display">
        <span className="wpm-value">{wpm}</span>
        <span className="wpm-label">WPM</span>
      </div>
      <div className="raw-wpm">
        Raw: {rawWpm}
      </div>
    </main>
  );
}

export default App;
