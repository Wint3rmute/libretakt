# Prezentacja 1 - opis projektu

**`Libretakt`** - kolaboratywne środowisko do produkcji muzyki.

# Libretakt - wstęp teoretyczny

![Sampler *Digitakt*, wyprodukowany przez szwecką firmę Elektron](./digitakt.png){ width=200px }

- odtwarzanie cyfrowych sampli,
- zmiana prędkości i kierunku odtwarzania, *loopowanie*
- nakładanie efektów na dźwięk,
- rytmiczna modulacja parametrów odtwarzania.

# Odtwarzanie/synteza dźwięku - PCM

!["*idealna*" sinusoida](./wave1.gif){width=150px}

![Przykładowa sinusoida wygenerowana komputerowo](./wave2.gif){width=150px}

Źródło: dokumentacja projektu [ALSA](https://www.alsa-project.org/alsa-doc/alsa-lib/pcm.html)


# Co wpływa na jakość dźwięku (i zużycie zasobów) podczas odtwarzania:

- Sampling rate - jak często generowana jest kolejna próbka
- Bit depth - z jaką dokładnością zapisujemy próbki

# Niskopoziomowy dostęp do karty dźwiękowej za pośrednictwem systemu operacyjnego

- Linux - `ALSA`/`PulseAudio`
- Android - `AAudio`/`OpenSL ES`
- MacOS/IOS - `CoreAudio`
- Aplikacje do produkcji muzyki - standard `VST`/`CLAP`


# Test Podświetlania Syntaxu Rust

```rust
fn main() {
    println!("Hello world");
}
```

