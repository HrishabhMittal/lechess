import torch
import torch.nn as nn
from torch.utils.data import DataLoader, random_split
from chessdata import ChessDataset

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

if __name__ == "__main__":
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    print(f"Training on {device}")
    
    dataset = ChessDataset("dataset/lichess-evals.csv")
    
    total_size = len(dataset)
    train_size = int(0.9 * total_size)
    val_size = total_size - train_size
    train_dataset, val_dataset = random_split(dataset, [train_size, val_size])
    print(f"Split: {train_size} training, {val_size} validation.")
    
    train_loader = DataLoader(train_dataset, batch_size=4096, shuffle=True, num_workers=8)
    val_loader = DataLoader(val_dataset, batch_size=4096, shuffle=False, num_workers=8)
    
    model = ChessNet().to(device)
    criterion = nn.MSELoss()
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
    scheduler = torch.optim.lr_scheduler.ReduceLROnPlateau(optimizer, mode='min', factor=0.3, patience=2)
    
    epochs = 30 
    
    for epoch in range(epochs):
        model.train() 
        total_train_loss = 0.0
        for batch_idx, (features, targets) in enumerate(train_loader):
            features, targets = features.to(device), targets.to(device)
            
            predictions = model(features)
            loss = criterion(predictions, targets)
            
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            
            total_train_loss += loss.item()
            
            if batch_idx % 50 == 0:
                print(f"Epoch {epoch+1} | Batch {batch_idx}/{len(train_loader)} | Loss: {loss.item():.4f}")
            
        model.eval() 
        total_val_loss = 0.0
        with torch.no_grad(): 
            for features, targets in val_loader:
                features, targets = features.to(device), targets.to(device)
                predictions = model(features)
                loss = criterion(predictions, targets)
                total_val_loss += loss.item()
                
        avg_train_loss = total_train_loss / len(train_loader)
        avg_val_loss = total_val_loss / len(val_loader)
        scheduler.step(avg_val_loss)
        
        print(f"=== EPOCH {epoch+1} COMPLETE | Train Loss: {avg_train_loss:.4f} | Val Loss: {avg_val_loss:.4f} ===")
        torch.save(model.state_dict(), f"chess_model.pth")
