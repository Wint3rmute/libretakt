FROM rust:latest

RUN apt-get update && apt-get install -y ffmpeg

COPY . .

RUN apt-get install -y libxcb-shape0-dev libxcb-xfixes0-dev libxcb1-dev libxkbcommon-dev libasound2-dev

RUN cargo run --example sample_stream | ffmpeg -f f32le -i pipe: -f mp3 - | ffmpeg -re -f mp3 -i pipe: -c copy -f flv rtmp://baczek.me/live/livestream