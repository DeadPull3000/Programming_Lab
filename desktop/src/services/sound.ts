// Lightweight cozy Web Audio API sound synthesizer
// Produces charming, soft organic retro-cozy audio without external audio files

class SoundService {
  private ctx: AudioContext | null = null;
  private enabled = true;

  constructor() {
    // AudioContext will be initialized on first user gesture
  }

  public setEnabled(val: boolean) {
    this.enabled = val;
  }

  public isEnabled(): boolean {
    return this.enabled;
  }

  private getContext(): AudioContext | null {
    if (!this.enabled) return null;
    if (!this.ctx && typeof window !== 'undefined') {
      const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
      if (AudioCtx) {
        this.ctx = new AudioCtx();
      }
    }
    if (this.ctx && this.ctx.state === 'suspended') {
      this.ctx.resume();
    }
    return this.ctx;
  }

  // Soft woodblock click on keypress / tile click
  public playClick() {
    const ctx = this.getContext();
    if (!ctx) return;

    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'triangle';
    osc.frequency.setValueAtTime(320, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(140, ctx.currentTime + 0.05);

    gain.gain.setValueAtTime(0.12, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.05);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.06);
  }

  // Soft bubble pop on tile selection or reveal
  public playPop(frequency = 480) {
    const ctx = this.getContext();
    if (!ctx) return;

    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(frequency * 0.8, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(frequency * 1.4, ctx.currentTime + 0.08);

    gain.gain.setValueAtTime(0.15, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.09);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.1);
  }

  // Gentle marimba chord for valid word submission
  public playSuccess() {
    const ctx = this.getContext();
    if (!ctx) return;

    const notes = [523.25, 659.25, 783.99, 1046.5]; // C5, E5, G5, C6
    notes.forEach((freq, idx) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();

      osc.type = 'sine';
      osc.frequency.setValueAtTime(freq, ctx.currentTime + idx * 0.06);

      gain.gain.setValueAtTime(0.001, ctx.currentTime + idx * 0.06);
      gain.gain.linearRampToValueAtTime(0.15, ctx.currentTime + idx * 0.06 + 0.02);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + idx * 0.06 + 0.35);

      osc.connect(gain);
      gain.connect(ctx.destination);

      osc.start(ctx.currentTime + idx * 0.06);
      osc.stop(ctx.currentTime + idx * 0.06 + 0.36);
    });
  }

  // Pangram discovery fanfare
  public playPangram() {
    const ctx = this.getContext();
    if (!ctx) return;

    const notes = [392.0, 523.25, 659.25, 783.99, 1046.5, 1318.51]; // G4, C5, E5, G5, C6, E6
    notes.forEach((freq, idx) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();

      osc.type = 'triangle';
      osc.frequency.setValueAtTime(freq, ctx.currentTime + idx * 0.07);

      gain.gain.setValueAtTime(0.16, ctx.currentTime + idx * 0.07);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + idx * 0.07 + 0.45);

      osc.connect(gain);
      gain.connect(ctx.destination);

      osc.start(ctx.currentTime + idx * 0.07);
      osc.stop(ctx.currentTime + idx * 0.07 + 0.46);
    });
  }

  // Soft low thud for invalid guess/error
  public playInvalid() {
    const ctx = this.getContext();
    if (!ctx) return;

    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(160, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(80, ctx.currentTime + 0.15);

    gain.gain.setValueAtTime(0.08, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.16);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.17);
  }

  // Realistic, cozy cat purr with rhythmic laryngeal tremor and soft greeting chirp
  public playPurr() {
    const ctx = this.getContext();
    if (!ctx) return;

    // 1. Gentle kitten chirrup / greeting mew at the start (0.24s)
    const chirpOsc = ctx.createOscillator();
    const chirpGain = ctx.createGain();
    chirpOsc.type = 'sine';
    chirpOsc.frequency.setValueAtTime(420, ctx.currentTime);
    chirpOsc.frequency.exponentialRampToValueAtTime(640, ctx.currentTime + 0.1);
    chirpOsc.frequency.exponentialRampToValueAtTime(490, ctx.currentTime + 0.22);

    chirpGain.gain.setValueAtTime(0.08, ctx.currentTime);
    chirpGain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.24);

    chirpOsc.connect(chirpGain);
    chirpGain.connect(ctx.destination);
    chirpOsc.start();
    chirpOsc.stop(ctx.currentTime + 0.25);

    // 2. Synthesize a 1.4s rhythmic cat purr buffer with ~25 Hz laryngeal modulation
    const duration = 1.4;
    const numSamples = Math.floor(ctx.sampleRate * duration);
    const buffer = ctx.createBuffer(1, numSamples, ctx.sampleRate);
    const data = buffer.getChannelData(0);

    for (let i = 0; i < numSamples; i++) {
      const t = i / ctx.sampleRate;

      // Two breath cycles (inhalation ~0.65s, exhalation ~0.75s)
      const breathPhase = (t / duration) * Math.PI * 2;
      const breathEnv = 0.55 + 0.45 * Math.sin(breathPhase);

      // Feline purr tremor modulation: 24-26 Hz laryngeal flutter
      const tremor = (Math.sin(2 * Math.PI * 25 * t) + 1) / 2;

      // Fundamental chest vibration: 85 - 95 Hz
      const f0 = 88 + 5 * Math.sin(breathPhase);
      const wave1 = Math.sin(2 * Math.PI * f0 * t);
      const waveSub = 0.45 * Math.sin(2 * Math.PI * (f0 * 0.5) * t);
      const waveHarm = 0.3 * Math.sin(2 * Math.PI * (f0 * 2) * t);

      // Subtle vocal airway air turbulence / breath friction
      const airNoise = (Math.random() * 2 - 1) * 0.08;

      // Smooth window envelope: fade in, sustain, gentle fade out
      const windowEnv = Math.sin(Math.PI * (i / numSamples));

      data[i] =
        (wave1 + waveSub + waveHarm + airNoise) *
        (0.25 + 0.75 * tremor) *
        breathEnv *
        windowEnv *
        0.32;
    }

    const purrSource = ctx.createBufferSource();
    purrSource.buffer = buffer;

    // Filter through warm lowpass filter to produce deep chest rumble
    const filter = ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(260, ctx.currentTime);
    filter.Q.value = 1.6;

    const purrGain = ctx.createGain();
    purrGain.gain.setValueAtTime(0.001, ctx.currentTime);
    purrGain.gain.linearRampToValueAtTime(0.38, ctx.currentTime + 0.1);
    purrGain.gain.setValueAtTime(0.38, ctx.currentTime + duration - 0.2);
    purrGain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + duration);

    purrSource.connect(filter);
    filter.connect(purrGain);
    purrGain.connect(ctx.destination);

    purrSource.start(ctx.currentTime + 0.04);
  }

  // Backward compatible alias
  public playMeow() {
    this.playPurr();
  }

  // Crisp paper page flip rustle
  public playPageFlip() {
    const ctx = this.getContext();
    if (!ctx) return;

    // Create a 0.18s white noise buffer for realistic paper friction
    const bufferSize = Math.floor(ctx.sampleRate * 0.18);
    const buffer = ctx.createBuffer(1, bufferSize, ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) {
      data[i] = (Math.random() * 2 - 1) * Math.exp(-i / (bufferSize * 0.35));
    }

    const noise = ctx.createBufferSource();
    noise.buffer = buffer;

    // Filter to paper rustle frequency
    const filter = ctx.createBiquadFilter();
    filter.type = 'bandpass';
    filter.frequency.setValueAtTime(1400, ctx.currentTime);
    filter.frequency.exponentialRampToValueAtTime(450, ctx.currentTime + 0.18);
    filter.Q.value = 1.8;

    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0.22, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.18);

    noise.connect(filter);
    filter.connect(gain);
    gain.connect(ctx.destination);

    noise.start();
  }
}

export const sound = new SoundService();
