from torch.utils.data import DataLoader, random_split, TensorDataset
import torch
import torch.nn as nn

class ChessNet(nn.Module):
    def __init__(self):
        super().__init__()
        self.fc1 = nn.Linear(772, 256) 
        self.relu = nn.ReLU()
        self.dropout = nn.Dropout(p=0.2)
        self.fc2 = nn.Linear(256, 1)
        self.sigmoid = nn.Sigmoid()
        
    def forward(self, x):
        x = self.relu(self.fc1(x))
        x = self.dropout(x)
        x = self.fc2(x)
        return self.sigmoid(x)

if __name__=="__main__":
    print("Loading binary tensors...")
    data = torch.load("dataset_tensors.pt", weights_only=False) 
    X = data['X']
    Y = data['Y']
    
    dataset = TensorDataset(X, Y)
    
    total_size = len(dataset)
    train_size = int(0.8 * total_size)
    val_size = total_size - train_size
    train_dataset, val_dataset = random_split(dataset, [train_size, val_size])
    print(f"split: {train_size} training, {val_size} validation.")
    
    train_loader = DataLoader(
        train_dataset, 
        batch_size=4096, 
        shuffle=True, 
        num_workers=0, 
        pin_memory=True
    )
    val_loader = DataLoader(
        val_dataset, 
        batch_size=4096, 
        shuffle=False, 
        num_workers=0, 
        pin_memory=True
    )
    
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"using device {device}")
    
    model = ChessNet().to(device)
    criterion = nn.MSELoss() 

    optimizer = torch.optim.Adam(model.parameters(), lr=0.001, weight_decay=1e-5)
    epochs = 10
    
    model.eval() 
    total_val_loss = 0.0
    with torch.no_grad(): 
        for features, targets in val_loader:
            features = features.to(device)
            targets = targets.to(device)
            
            predictions = model(features)
            loss = criterion(predictions, targets)
            total_val_loss += loss.item()
            
    avg_val_loss = total_val_loss / len(val_loader)
    print(f"epoch 0/{epochs} | val loss: {avg_val_loss:.4f}")
    for epoch in range(epochs):
        model.train() 
        total_train_loss = 0.0
        for batch_idx, (features, targets) in enumerate(train_loader):
            features = features.to(device)
            targets = targets.to(device)
            
            predictions = model(features)
            loss = criterion(predictions, targets)
            
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            
            total_train_loss += loss.item()
            
        model.eval() 
        total_val_loss = 0.0
        with torch.no_grad(): 
            for features, targets in val_loader:
                features = features.to(device)
                targets = targets.to(device)
                
                predictions = model(features)
                loss = criterion(predictions, targets)
                total_val_loss += loss.item()
                
        avg_train_loss = total_train_loss / len(train_loader)
        avg_val_loss = total_val_loss / len(val_loader)
        print(f"epoch {epoch+1}/{epochs} | train loss: {avg_train_loss:.4f} | val loss: {avg_val_loss:.4f}")
        
    torch.save(model.state_dict(), "chess_model.pth")
    print("training complete")
