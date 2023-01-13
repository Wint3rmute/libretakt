FROM python:3.10

RUN apt update && apt install ffmpeg libsndfile1-dev libsndfile1 -y

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt

COPY genre-classificator /app/genre-classificator
WORKDIR /app/genre-classificator

