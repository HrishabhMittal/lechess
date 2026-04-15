import torch
from torch.utils.data import Dataset
import pandas as pd
import numpy as np
import chess
class ChessDataset(Dataset):
    def __init__(self, csv_file):
        self.data = pd.read_csv(csv_file, names=['FEN', 'Analysis'], header=0)

    def __len__(self):
        return len(self.data)

    def fen_to_features(self, fen):
        board = chess.Board(fen)
        features = np.zeros(772, dtype=np.float32)
        is_white_turn = (board.turn == chess.WHITE)

        for sq in chess.SQUARES:
            piece = board.piece_at(sq)
            if piece:
                mapped_sq = sq if is_white_turn else sq ^ 56
                cur_piece = (piece.color == board.turn)
                color_offset = 0 if cur_piece else 6
                piece_idx = piece.piece_type - 1
                index = (color_offset + piece_idx) * 64 + mapped_sq
                features[index] = 1.0

        if is_white_turn:
            if board.has_kingside_castling_rights(chess.WHITE): features[768] = 1.0
            if board.has_queenside_castling_rights(chess.WHITE): features[769] = 1.0
            if board.has_kingside_castling_rights(chess.BLACK): features[770] = 1.0
            if board.has_queenside_castling_rights(chess.BLACK): features[771] = 1.0
        else:
            if board.has_kingside_castling_rights(chess.BLACK): features[768] = 1.0
            if board.has_queenside_castling_rights(chess.BLACK): features[769] = 1.0
            if board.has_kingside_castling_rights(chess.WHITE): features[770] = 1.0
            if board.has_queenside_castling_rights(chess.WHITE): features[771] = 1.0
        return features

    def normalize_score(self, cp_score):
        return 1.0 / (1.0 + np.exp(-cp_score / 400.0))

    def __getitem__(self, idx):
        row = self.data.iloc[idx]
        fen = row['FEN']
        raw_score = float(row['Analysis'])

        features = self.fen_to_features(fen)
        features_tensor = torch.tensor(features, dtype=torch.float32)

        target_score = self.normalize_score(raw_score)
        target_tensor = torch.tensor([target_score], dtype=torch.float32)

        return features_tensor, target_tensor
