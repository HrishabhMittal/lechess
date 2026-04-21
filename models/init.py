import torch
from export import ChessNet, export_quantized
if __name__ == "__main__":
    model = ChessNet()
    torch.save(model.state_dict(), "models/chess_model.pth")
    export_quantized("models/chess_model.pth", "models/weights.msgpack")
