import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
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

  return (
    <main className="widget-container" data-tauri-drag-region>
      <div className="drag-region" data-tauri-drag-region></div>
      <div className="wpm-display" data-tauri-drag-region>
        <span className="wpm-value" data-tauri-drag-region>{wpm}</span>
        <span className="wpm-label" data-tauri-drag-region>WPM</span>
      </div>
      <div className="raw-wpm" data-tauri-drag-region>
        Raw: {rawWpm}
      </div>
    </main>
  );
}

export default App;
