import torch
import json
import numpy as np

def export_quantized(model_path, scale=127):
    checkpoint = torch.load(model_path, map_location=torch.device('cpu'))
    quantized_weights = {}
    for name, param in checkpoint.items():
        q_param = (param * scale).round().to(torch.int16)
        quantized_weights[name] = q_param.numpy().tolist()
    with open("weights.json", "w") as f:
        json.dump(quantized_weights, f)

if __name__ == "__main__":
    export_quantized("chess_model.pth")
