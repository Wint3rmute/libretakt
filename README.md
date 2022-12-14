# Libretakt

An open-source standalone software sampler, allowing for collaborative music creation over the network & streaming your performance via an internet radio.

Workflow and project name inspired by Elektron's [Digitakt](https://www.elektron.se/us/digitakt-explorer).

![UI screenshot](./ui_screenshot.jpg)

## Features

- 4 voice polyphony
- Elektron-style parameter locking
- Every voice gets a separate:
  - Amp ADSR
  - Filter ADSR
  - Delay effect with adjustable send, feedback & length
  - Reverb effect with adjustable send, size & early mix
  - [TODO] an LFO
- [TODO] a master compressor


## Running

1. `cargo run` - default sampler UI with audio engine, no synchronisation server.
2. `cargo run --features enable_synchronisation` - same as above, synchronisation enabled. **Remember to start the server first**.
3. `cargo run -p server` - starts the synchronisation server.
4. `cargo run --example headless_ffmpeg_client | ffmpeg -f f32le -i pipe: -f mp3 - | ffmpeg -re -f mp3 -i pipe: -c copy -f flv rtmp://baczek.me/live/livestream` - Headless streaming client. Again, **remember to start the server first**

---

Note: below are the notes collected during development and some diagrams we made for the university course.
Most of it is messy, some notes are in polish :)

## Learning resources used during development

- Audio playback - [Rodio](https://github.com/RustAudio/rodio)
- UI - [MacroQuad](https://macroquad.rs/)
- Music genre classification:
  - [python tools overview](https://farranaanjum05.medium.com/music-genre-classification-with-python-51bff77adfd6)
  - [GTZAN dataset](https://www.kaggle.com/datasets/andradaolteanu/gtzan-dataset-music-genre-classification)
- On mutexes and audio processing - [using locks in real-time audio processing](Using locks in real-time audio processing, safely)

## Component diagram

```mermaid
flowchart TD
    subgraph Client
        cSeq[Sequencer]--send playback events-->cEng
        cEng[DSP engine]
        cCont[Controller]--apply state change-->cSeq
        cUI[UI]--read state-->cSeq
        cUI--send state change<br>join/create session-->cCont
    end

    cCont--send state change<br>join/create session-->sync
    sync--synchronise state<br>with other uses-->cCont

    subgraph Web-hosted
        direction RL;
        subgraph hC[Headless client - spawned per session]
            Controller--apply state change-->Sequencer
            Sequencer--send playback events-->e[DSP engine]
        end
        sync[Synchronisation<br>service]
        sync--send state change-->Controller
        e-->stream
        stream[Streaming service]
    end
```

## Class diagram

```mermaid
classDiagram
    class Engine {
        sequencer: SmartPointer~Sequencer~
        voices: Vec[Voice]

        next() Rodio requirement
    }

    Engine --> Sequencer: Engine calls tick() in the sequencer, <br>triggering events and passing its voices

    class Sequencer {
        tracks: Vec[Track]
        tick(&mut voices)
    }

    class UI {
        sequencer: SmartPointer~Sequencer~
        controller: SmartPointer~Controller~
        draw()
    }

    class Controller {
        sequencer: SmartPointer~Sequencer~
        websocket_worker: Thread? idk
        set_parameter(track, pattern, step_num, step)
    }

    Controller -->Sequencer: Controller mutates the sequencer state,<br>either because of user input or because<br>of received websocket events
    Controller <-- UI: Handle parameter change events
    UI --> Sequencer: UI reads the sequencer state

    class Track {
        playback_parameters: PlaybackParameters
        patterns: Vec[Pattern],
        current_pattern: int,
        current_step: int,
    }

    Track *-- Pattern : Patterns refer to different melodies, that <br>can be dynamically switched by users

    class Pattern {
        steps: Vec[Step]
    }

    Pattern *-- Step: Step plays a sound, optionally<br> overriding playback parameters

    class PlaybackParameters {
        parameters: [u8]

        merge(Step) PlaybackParameters
    }

    Track *-- PlaybackParameters

    class Step {
        parameters: [Option~u8~]
    }

    Sequencer *-- Track

    class Voice {
        sample_provider: SampleProvider
        play_position: float
        playback_parameters: PlaybackParameters
    }

    Voice *-- PlaybackParameters
    Engine *-- Voice

    SynchronisationService <--> Controller: Synchronise changes across users<br>within the same session

    class SynchronisationService {
        sessions: Map~session_token: String, connected_clients: WebsocketConnections~
        join_or_create_session(token, nickname)
        set_parameter(track, pattern, step_num, step)
        parameter_changes_subscription(): Stream of updates(track, pattern, step_num, step)
    }
```


## Gantt

```mermaid
gantt
    title Rozk??ad jazduni
    dateFormat  YYYY-MM-DD

    section Web
    POC serwer websocket    :w1, 2022-10-15, 7d
    POC serializacja&deserializacja Cap'n'Proto  : 2022-10-15, 7d
    Testy synchronizacji stanu za pomoc?? Cap'n'Proto  : 7d

    section Logika biznesowa
    Mechanizm synchronizacji stanu :crit, 2022-10-15, 7d
    Zapisywanie stanu sequencera: 7d
    Blockowanie ??cie??ek na jednego u??ytkownika: 7d

    section UI
    Widgety z wizualizacj?? parametr??w   :crit, 2022-10-15, 7d
    Podpi??cie sequencera do widget??w   :crit, 7d
    Podgl??d i edycja sekwencji : 7d
    Obs??uga parameter locks : 7d
    Obs??uga prze????czania ??cie??ek i pattern??w : 7d

    section DSP&Sequencing
    Filtry na audio :2022-10-15, 7d
    Efekty reverb + delay : 7d
    (AMP + Filter) ADSR: 7d
```



## Do rozpisania

1. Wymagania funkcjonalne i niefunkcjnalne:
  - Zdefiniowanie czemu dana technologia zosta??a wybrana ("najlepiej spe??nia wymagania XXX poniewa?? YYY")
2. Zamodelowanie proces??w oddzia??ywania u??ytkownika z systemem
3. Opis komunikacji mi??dzy komponentami w systemie
4. Rozmieszczenie komponentw systemu


### Gotowy plan raportu z wyk??adu xd

#### Wst??p

1. Opis rzeczywisto??ci, w kt??rej funkcjonuje system
2. Opis klas obiekt??w wyst??puj??cych w rzeczywisto??ci (modelowanie)
3. Opis atrybut??w tych??e klas
4. Opis relacji

#### Opis zada??

1. Diagram przypadk??w u??ycia
2. Flow chart
3. Sequence diagram


#### Opis proces??w

"Proces biznesowy" - opisanie jakie procesy realizuje aplikacja z perspektywy u??ytkownika

#### Opis wymiany komunikat??w

M??wi?? o tym tak, jakby by??o dla niego wa??ne.

1. Opis/definicja wiadomo??ci
2. Opis sekwencji wysy??ania wiadomo??ci


#### Opis rozmieszczenia komponent??w

VPS/VM/Dockerki, inne takie



## Old diagram

```mermaid
graph TD
    SEQ[Sequencer]
    SOUND[Sound Engine]
    UI[User interface] -->|Read state| SEQ
    UI-->|Edit state| SEQ
    MQ[Message queue]
    SEQ-->|Emit trigger| SOUND
    MQ-->|Propagate state changes from other users|SEQ
    UI-->|Send edit state event|MQ
    SOUND-->|sample state visualisation|UI
```

![](architecture_prototype.drawio.svg)



