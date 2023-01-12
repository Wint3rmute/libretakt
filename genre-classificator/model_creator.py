import pandas as pd
import pickle

from sklearn import preprocessing
from sklearn.metrics import accuracy_score
from sklearn.model_selection import train_test_split
from xgboost import XGBClassifier

# use features_3_sec file from GTZAN dataset
data = pd.read_csv('data/features_3_sec.csv')
data = data.iloc[0:, 1:]

y = data['label']  # genre variable.
X = data.loc[:, data.columns != 'label']  # select all columns except labels


# normalize data
cols = X.columns
min_max_scaler = preprocessing.MinMaxScaler()
np_scaled = min_max_scaler.fit_transform(X)
X = pd.DataFrame(np_scaled, columns=cols)

# index labels
labels = ["blues", "classical", "country", "disco", "hiphop", "jazz", "metal", "pop", "reggae", "rock"]
y = [labels.index(y_) for y_ in y]

# split data
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.3, random_state=42)

# model
xgb = XGBClassifier(n_estimators=1000, learning_rate=0.05)
xgb.fit(X_train, y_train)
pickle.dump(xgb, open('model', "wb"))

print(f'Accuracy: {round(accuracy_score(y_test, xgb.predict(X_test)), 5)}')
