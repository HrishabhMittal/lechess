import torch
from torch.utils.data import DataLoader
import chessdata as ds
from tqdm import tqdm
import os

if __name__ == '__main__':
    dataset = ds.ChessDataset("dataset/big_dataset.csv")
    
    num_cores = os.cpu_count() or 4
    print(f"{num_cores} cpu workers")

    loader = DataLoader(
        dataset, 
        batch_size=8192, 
        shuffle=False, 
        num_workers=num_cores
    )

    all_features = []
    all_targets = []

    for features, targets in tqdm(loader, total=len(loader)):
        all_features.append(features)
        all_targets.append(targets)

    X = torch.cat(all_features, dim=0)
    Y = torch.cat(all_targets, dim=0)

    torch.save({'X': X, 'Y': Y}, "dataset_tensors.pt")
