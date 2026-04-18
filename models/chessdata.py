import torch
from torch.utils.data import Dataset
import pandas as pd
import numpy as np

class ChessDataset(Dataset):
    def __init__(self, csv_file):
        self.data = pd.read_csv(csv_file, names=['FEN', 'Evaluation'], header=0)

    def __len__(self):
        return len(self.data)

    def fast_fen_to_halfkp(self, fen):
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

    def parse_eval(self, eval_str, is_white_turn):
        eval_str = str(eval_str)
        if '#' in eval_str:
            mate_in = int(eval_str.replace('#', ''))
            val = 10000 if mate_in > 0 else -10000
        else:
            val = int(eval_str)
            
        if not is_white_turn:
            val = -val
            
        return 1.0 / (1.0 + np.exp(-val / 400.0))

    def __getitem__(self, idx):
        row = self.data.iloc[idx]
        fen = row['FEN']
        eval_val = row['Evaluation']
        
        is_white_turn = fen.split(' ')[1] == 'w'
        
        features = self.fast_fen_to_halfkp(fen)
        target = self.parse_eval(eval_val, is_white_turn)
        
        return torch.tensor(features), torch.tensor([target], dtype=torch.float32)
