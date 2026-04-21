import torch
import torch.nn as nn
from torch.utils.data import Dataset, DataLoader
import torch.multiprocessing
import numpy as np
class ChessNetRL(nn.Module):
    def __init__(self):
        super().__init__()
        self.fc1 = nn.Linear(45056, 512) 
        self.fc2 = nn.Linear(512, 32)
        self.fc3 = nn.Linear(32, 32)
        self.fc4 = nn.Linear(32, 1)
    def forward(self, x):
        x = torch.clamp(self.fc1(x), 0.0, 1.0)
        x = torch.clamp(self.fc2(x), 0.0, 1.0)
        x = torch.clamp(self.fc3(x), 0.0, 1.0)
        x = self.fc4(x)
        return torch.sigmoid(x)

def fast_fen_to_halfkp(fen):
    parts = fen.split(' ')
    board_part = parts[0]
    is_white_turn = (parts[1] == 'w')
    
    features = np.zeros(45056, dtype=np.float32)
    pieces = []
    friendly_king_sq = -1
    
    rank = 7
    file = 0
    for c in board_part:
        if c == '/':
            rank -= 1
            file = 0
        elif c.isdigit():
            file += int(c)
        else:
            sq = rank * 8 + file
            is_white_piece = c.isupper()
            p_char = c.lower()
            is_friendly = (is_white_piece == is_white_turn)
            
            if p_char == 'p': pt = 0
            elif p_char == 'n': pt = 1
            elif p_char == 'b': pt = 2
            elif p_char == 'r': pt = 3
            elif p_char == 'q': pt = 4
            elif p_char == 'k': pt = 5
            
            if pt == 5 and is_friendly:
                friendly_king_sq = sq
            else:
                pieces.append((sq, is_friendly, pt))
            
            file += 1
            
    if not is_white_turn:
        friendly_king_sq ^= 56
        
    for sq, is_friendly, pt in pieces:
        mapped_sq = sq if is_white_turn else sq ^ 56
        
        if is_friendly:
            p_idx = pt 
        else:
            p_idx = pt + 5 
        
        idx = friendly_king_sq * 704 + p_idx * 64 + mapped_sq
        features[idx] = 1.0
        
    return features
class RLDataset(Dataset):
    def __init__(self, file_path):
        self.fens = []
        self.search_scores = []
        self.game_results = []
        print(f"loading from {file_path}...")
        
        with open(file_path, 'r') as f:
            for line in f:
                parts = line.strip().split(' | ')
                if len(parts) == 3:
                    fen, score_cp, result = parts
                    self.fens.append(fen)
                    self.search_scores.append(float(score_cp))
                    self.game_results.append(float(result))
        self.search_scores = torch.tensor(self.search_scores, dtype=torch.float32)
        self.game_results = torch.tensor(self.game_results, dtype=torch.float32)

    def __len__(self):
        return len(self.fens)

    def __getitem__(self, idx):
        fen_str = self.fens[idx]
        feature_array = fast_fen_to_halfkp(fen_str) 
        features_tensor = torch.tensor(feature_array, dtype=torch.float32)
        
        return features_tensor, self.search_scores[idx], self.game_results[idx]

if __name__ == "__main__":
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    print(f"training on {device}")
    dataset = RLDataset("self_play_data.txt")
    dataloader = DataLoader(dataset, batch_size=8192, shuffle=True, num_workers=0)
    model = ChessNetRL().to(device)
    try:
        model.load_state_dict(torch.load("models/chess_model.pth"))
        print("loaded previous weights")
    except:
        print("init with random weights")
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
    criterion = nn.MSELoss()
    LAMBDA = 0.5 
    epochs = 15
    for epoch in range(epochs):
        model.train()
        total_loss = 0.0
        for batch_idx, (features, search_scores, game_results) in enumerate(dataloader):
            features = features.to(device)
            search_scores = search_scores.to(device)
            game_results = game_results.to(device)
            search_probs = torch.sigmoid(search_scores / 400.0)
            blended_targets = (LAMBDA * game_results) + ((1.0 - LAMBDA) * search_probs)
            blended_targets = blended_targets.unsqueeze(1) 
            predictions = model(features)
            loss = criterion(predictions, blended_targets)
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            total_loss += loss.item()
            print(f"epoch {epoch+1} | batch {batch_idx}/{len(dataloader)} | loss: {loss.item():.4f}")
        print(f"epoch {epoch+1} | loss: {total_loss / len(dataloader):.5f}")
        torch.save(model.state_dict(), "models/chess_model.pth")
