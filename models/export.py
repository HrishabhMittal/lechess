import torch
import json
from main import ChessNet

model = ChessNet()
model.load_state_dict(torch.load("chess_model.pth", weights_only=True))
model.eval()

weights = {
    "fc1_weight": model.fc1.weight.detach().numpy().tolist(),
    "fc1_bias": model.fc1.bias.detach().numpy().tolist(),
    
    "fc2_weight": model.fc2.weight.detach().numpy().tolist(),
    "fc2_bias": model.fc2.bias.detach().numpy().tolist(),
    
    "fc3_weight": model.fc3.weight.detach().numpy().tolist(),
    "fc3_bias": model.fc3.bias.detach().numpy().tolist(),
    
    "fc4_weight": model.fc4.weight.detach().numpy().tolist(),
    "fc4_bias": model.fc4.bias.detach().numpy().tolist(),
}
with open("weights.json", "w") as f:
    json.dump(weights, f)

print("exported weights.json")
