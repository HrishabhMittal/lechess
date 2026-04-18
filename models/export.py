import json
import torch
import torch.nn as nn
class ChessNet(nn.Module):
    def __init__(self):
        super().__init__()
        self.fc1 = nn.Linear(45056, 256) 
        self.fc2 = nn.Linear(256, 32)
        self.fc3 = nn.Linear(32, 1)
        
    def forward(self, x):
        x = torch.clamp(self.fc1(x), 0.0, 1.0)
        x = torch.clamp(self.fc2(x), 0.0, 1.0)
        x = self.fc3(x)
        return torch.sigmoid(x)
def export_quantized(model_path, export_path, scale=127):
    model = ChessNet()
    model.load_state_dict(torch.load(model_path, map_location=torch.device('cpu')))

    quantized_weights = {}
    for name, param in model.state_dict().items():
        q_param = (param * scale).round().to(torch.int16)
        quantized_weights[name] = q_param.numpy().tolist()

    with open(export_path, "w") as f:
        json.dump(quantized_weights, f)
    print(f"Exported quantized weights to {export_path}")

export_quantized(
    "chess_model_halfkp.pth",
    "weights.json"
)
