// V7 push-to-talk dictation (SPEC §14.3, plan §3 e6). Press-and-hold to dictate;
// releasing stops. Prefers the on-device browser SpeechRecognition (nothing leaves
// the device); falls back to the Assist STT endpoint only when the browser has no
// recogniser AND the `dictation` capability is granted — in which case the audio is
// covered by the "what left the device" disclosure. Transcribed text is handed to
// `onTranscript`; it is NEVER auto-sent.

import { createSignal, Show, onCleanup, onMount, type JSX } from 'solid-js';
import { AssistService } from './service.ts';
import { hasCapability, type AssistConfig } from './types.ts';
import {
  browserRecognitionCtor,
  mediaRecorderSupported,
  transcriptFromEvent,
  type SpeechRecognitionLike,
} from './dictation.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

export interface DictationProps {
  config: AssistConfig;
  service: AssistService;
  /** Append recognised text to the field being dictated into. */
  onTranscript: (text: string) => void;
  /** BCP-47 language tag for the recogniser. */
  lang?: string;
}

type Source = 'browser' | 'endpoint' | 'none';

export function Dictation(props: DictationProps): JSX.Element {
  onMount(() => void loadCatalog('assist'));
  const source = (): Source => {
    if (browserRecognitionCtor() !== null) return 'browser';
    if (mediaRecorderSupported() && hasCapability(props.config, 'dictation')) return 'endpoint';
    return 'none';
  };

  const [recording, setRecording] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let recognition: SpeechRecognitionLike | null = null;
  let recorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];
  let stream: MediaStream | null = null;

  function stopStream(): void {
    stream?.getTracks().forEach((t) => t.stop());
    stream = null;
  }
  onCleanup(() => {
    recognition?.stop();
    if (recorder?.state === 'recording') recorder.stop();
    stopStream();
  });

  function startBrowser(): void {
    const Ctor = browserRecognitionCtor();
    if (Ctor === null) return;
    recognition = new Ctor();
    recognition.lang = props.lang ?? 'en-US';
    recognition.interimResults = false;
    recognition.continuous = true;
    recognition.onresult = (event) => {
      const text = transcriptFromEvent(event);
      if (text.length > 0) props.onTranscript(text);
    };
    recognition.onerror = () => setError(t('assist-dictate-err'));
    recognition.onend = () => setRecording(false);
    recognition.start();
    setRecording(true);
  }

  async function startEndpoint(): Promise<void> {
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      chunks = [];
      recorder = new MediaRecorder(stream);
      recorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunks.push(e.data);
      };
      recorder.onstop = () => {
        stopStream();
        const blob = new Blob(chunks, { type: recorder?.mimeType ?? 'audio/webm' });
        void props.service
          .transcribe(blob)
          .then((text) => {
            if (text.length > 0) props.onTranscript(text);
          })
          .catch(() => setError(t('assist-transcribe-err')));
      };
      recorder.start();
      setRecording(true);
    } catch {
      setError(t('assist-mic-err'));
      setRecording(false);
    }
  }

  function start(): void {
    setError(null);
    if (recording()) return;
    if (source() === 'browser') startBrowser();
    else if (source() === 'endpoint') void startEndpoint();
  }

  function stop(): void {
    if (source() === 'browser') recognition?.stop();
    else if (recorder?.state === 'recording') recorder.stop();
    setRecording(false);
  }

  return (
    <Show when={source() !== 'none'}>
      <div class={css.row} data-module="assist-dictation">
        <button
          type="button"
          class={recording() ? `${css.mic} ${css.micActive}` : css.mic}
          aria-pressed={recording()}
          aria-label={recording() ? t('assist-dictate-stop') : t('assist-dictate-hold')}
          onPointerDown={() => start()}
          onPointerUp={() => stop()}
          onPointerLeave={() => recording() && stop()}
        >
          {recording() ? t('assist-dictate-listening') : t('assist-dictate-hold-text')}
        </button>
        <Show when={source() === 'endpoint'}>
          <span class={css.meta}>{t('assist-dictate-endpoint-note')}</span>
        </Show>
        <Show when={error() !== null}>
          <span class={css.error} role="alert">
            {error()}
          </span>
        </Show>
      </div>
    </Show>
  );
}
