import { useRef, useCallback, useEffect } from "react";

export function useSineWave() {
  const audioCtxRef = useRef<AudioContext | null>(null);
  const oscRef = useRef<OscillatorNode | null>(null);
  const isPlayingRef = useRef<boolean>(false);

  const start = useCallback((frequency: number = 1000) => {
    if (isPlayingRef.current) return;

    if (!audioCtxRef.current || audioCtxRef.current.state === "closed") {
      audioCtxRef.current = new (
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext })
          .webkitAudioContext
      )();
    }

    const ctx = audioCtxRef.current;
    if (ctx.state === "suspended") {
      ctx.resume();
    }

    const oscNode = ctx.createOscillator();
    oscNode.type = "sine";
    oscNode.frequency.setValueAtTime(frequency, ctx.currentTime);
    oscNode.connect(ctx.destination);
    oscNode.start();

    oscRef.current = oscNode;
    isPlayingRef.current = true;
  }, []);

  const stop = useCallback(() => {
    if (!isPlayingRef.current || !oscRef.current) return;

    try {
      oscRef.current.stop();
    } catch {}
    oscRef.current.disconnect();
    oscRef.current = null;
    isPlayingRef.current = false;
  }, []);

  useEffect(() => {
    return () => {
      if (oscRef.current) {
        try {
          oscRef.current.stop();
        } catch {}
        oscRef.current.disconnect();
      }
      if (audioCtxRef.current && audioCtxRef.current.state !== "closed") {
        audioCtxRef.current.close();
      }
    };
  }, []);

  return { start, stop };
}
