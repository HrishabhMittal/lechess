import torch
import numpy as np
import chessdata as ds
from tqdm import tqdm

dataset = ds.ChessDataset("dataset/dataset.csv")
all_features = []
all_targets = []

for i in tqdm(range(len(dataset))):
    features, target = dataset[i]
    all_features.append(features)
    all_targets.append(target)

X = torch.stack(all_features)
Y = torch.stack(all_targets)

torch.save({'X': X, 'Y': Y}, "dataset_tensors.pt")
