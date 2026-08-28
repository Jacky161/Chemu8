import { useEffect, useRef, useState, type ChangeEvent } from "react";
import "./App.css";
import { Chip8Wasm } from "backend";
import { useSineWave } from "./useSineWave";

const C8_WIDTH = 64;
const C8_HEIGHT = 32;
const SCALE = 15;
const CANVAS_WIDTH = C8_WIDTH * SCALE;
const CANVAS_HEIGHT = C8_HEIGHT * SCALE;
const CANVAS_ID = "canvas";

const FRAMERATE = 60;
const FPS_INTERVAL = 1000 / FRAMERATE;
const TICKS_PER_FRAME = 10;
const BEEP_FREQUENCY = 1000;

function App() {
  const [romLoaded, setRomLoaded] = useState(false);
  const chip8Ref = useRef<Chip8Wasm | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const { start: startSineWave, stop: stopSineWave } = useSineWave();

  const [quirkShifting, setQuirkShifting] = useState(true);
  const [quirkMemory, setQuirkMemory] = useState(true);

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const requestAnimFrameRef = useRef<number | null>(null);
  const lastFrameTimeRef = useRef(0);

  // Handle when a user selects a file
  const handleFileChange = (event: ChangeEvent<HTMLInputElement>) => {
    const chip8 = chip8Ref.current;
    const files = event.target.files;
    setRomLoaded(false);
    if (!files || files.length <= 0 || !chip8) return;

    // Cancel rendering the last ROM
    if (requestAnimFrameRef.current) {
      window.cancelAnimationFrame(requestAnimFrameRef.current);
    }

    const reader = new FileReader();
    reader.onload = (e) => {
      const buffer = e.target?.result;
      if (!(buffer instanceof ArrayBuffer)) return;

      const rom = new Uint8Array(buffer);
      chip8.reset()
      chip8.set_quirk_shifting(quirkShifting);
      chip8.set_quirk_memory(quirkMemory);
      chip8.load_game(rom)
      setRomLoaded(true);
    };

    reader.readAsArrayBuffer(files[0]);
  };

  // Initialisation on Page Load
  useEffect(() => {
    chip8Ref.current = new Chip8Wasm(CANVAS_ID);
    const chip8 = chip8Ref.current;
    const keyDownListener = (evt: KeyboardEvent) => {
      chip8.keypress(evt, true);
    }
    const keyUpListener = (evt: KeyboardEvent) => {
      chip8.keypress(evt, false);
    }
    document.addEventListener("keydown", keyDownListener);
    document.addEventListener("keyup", keyUpListener);

    // Sync quirks with chip8 instance
    setQuirkShifting(chip8.quirk_shifting());
    setQuirkMemory(chip8.quirk_memory());

    return () => {
      // Cleanup function
      document.removeEventListener("keydown", keyDownListener);
      document.removeEventListener("keyup", keyUpListener);
    };
  }, []);

  // Start main loop when ROM loads
  useEffect(() => {
    if (!romLoaded) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const chip8 = chip8Ref.current;
    if (!chip8) return;

    /*
     * requestAnimationFrame runs at monitor refresh rate, but need to cap to 60FPS.
     * Source - https://stackoverflow.com/a/19772220
     * Posted by markE, modified by community. See post 'Timeline' for change history
     * Retrieved 2026-08-26, License - CC BY-SA 3.0
     */
    const mainLoop = () => {
      requestAnimFrameRef.current = window.requestAnimationFrame(mainLoop);

      const now = Date.now()
      const deltaTime = now - lastFrameTimeRef.current;

      // Do not render if it hasn't been long enough between frames.
      if (deltaTime <= FPS_INTERVAL) return;
      lastFrameTimeRef.current = now - (deltaTime % FPS_INTERVAL);

      for (let i = 0; i < TICKS_PER_FRAME; i++) {
        chip8.tick()
      }
      const shouldBeep = chip8.tick_timers()

      if (shouldBeep) {
        startSineWave(BEEP_FREQUENCY);
      } else {
        stopSineWave();
      }

      // Clear the canvas before drawing
      ctx.fillStyle = "black"
      ctx.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)

      ctx.fillStyle = "white"
      chip8.draw_screen(SCALE)
      chip8.notify_vblank();
    }

    requestAnimFrameRef.current = window.requestAnimationFrame(mainLoop);

    return () => {
      if (requestAnimFrameRef.current !== null) {
        window.cancelAnimationFrame(requestAnimFrameRef.current);
      }
    }
  }, [romLoaded])

  // Update backend when quirk is changed
  useEffect(() => {
    chip8Ref.current?.set_quirk_shifting(quirkShifting);
    chip8Ref.current?.set_quirk_memory(quirkMemory);
  }, [quirkShifting, quirkMemory])

  return (
    <>
      <section id="center">
        <div>
          <h1>Chemu8</h1>
        </div>
        <canvas
          id={CANVAS_ID}
          ref={canvasRef}
          width={CANVAS_WIDTH}
          height={CANVAS_HEIGHT}
          style={{ backgroundColor: "black" }}
        />
        <input
          type="file"
          ref={fileInputRef}
          onChange={handleFileChange}
          style={{ display: 'none' }}
          accept=".ch8"
        />

        <div className="controls-panel">
          <div className="action-row">
            <button
              type="button"
              className="button primary-btn"
              onClick={() => fileInputRef.current?.click()}
            >
              Load ROM
            </button>
          </div>

          <fieldset className="settings-card">
            <legend className="settings-legend">Quirk Options</legend>

            <div className="quirks-row">
              <label className="quirk-toggle">
                <input
                  type="checkbox"
                  name="quirkShifting"
                  checked={quirkShifting}
                  onChange={(event) => setQuirkShifting(event.target.checked)}
                />
                <span className="toggle-label">Shifting</span>
              </label>

              <label className="quirk-toggle">
                <input
                  type="checkbox"
                  name="quirkMemory"
                  checked={quirkMemory}
                  onChange={(event) => setQuirkMemory(event.target.checked)}
                />
                <span className="toggle-label">Memory</span>
              </label>
            </div>
          </fieldset>
        </div>

      </section>
    </>
  );
}

export default App;
