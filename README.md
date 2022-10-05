# Libretakt

Project name based on Elektron's [Digitakt](https://www.elektron.se/us/digitakt-explorer) (but free/libre/etc).

## Learning resources

- Audio playback - [Rodio](https://github.com/RustAudio/rodio)
- UI - [MacroQuad](https://macroquad.rs/)

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

