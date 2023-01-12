
import pickle
import time

import librosa
import numpy as np
import requests
from librosa.beat import tempo
from librosa.effects import harmonic, percussive
from librosa.feature import chroma_stft, rms, spectral_centroid, spectral_bandwidth, spectral_rolloff, \
    zero_crossing_rate, mfcc

# load model
model = pickle.load(open('model', "rb"))
labels = ["blues", "classical", "country", "disco", "hiphop", "jazz", "metal", "pop", "reggae", "rock"]

while True:
    # download sample
    body = []
    start = time.time()
    r = requests.get('http://70.34.252.191:8080/live/livestream.flv', verify=False, stream=True)

    for chunk in r.iter_content():
        body.append(chunk)

        if time.time() > (start + 3):
            break

    with open("sample.flv", "wb") as binary_file:
        binary_file.write(b''.join(body))

    # open sample
    y, sr = librosa.load("sample.flv")
    audio, _ = librosa.effects.trim(y)

    # extract features
    features = [len(audio)]

    #  basic
    for function in [chroma_stft, rms, spectral_centroid, spectral_bandwidth, spectral_rolloff, zero_crossing_rate,
                     harmonic, percussive]:
        feature = function(y=audio)
        features.append(np.mean(feature))
        features.append(np.var(feature))

    #  tempo
    features.append(tempo(y=audio)[0])

    #  mfcc's
    mfcc_list = mfcc(y=audio, n_mfcc=20)
    for mfcc_ in mfcc_list:
        features.append(np.mean(mfcc_))
        features.append(np.var(mfcc_))

    # predict
    features = np.array(features).reshape(1, 58)

    prediction_index = int(model.predict(features))

    print(labels[prediction_index])
