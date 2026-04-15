.PHONY: all dataset train m_export clean

all: m_export

dataset_tensors.pt: preprocess.py chessdata.py dataset/dataset.csv
	python preprocess.py

dataset: dataset_tensors.pt


chess_model.pth: main.py chessdata.py dataset_tensors.pt
	python main.py

train: chess_model.pth


weights.json: export.py main.py chess_model.pth
	python export.py

m_export: weights.json

clean:
	rm -f dataset_tensors.pt chess_model.pth weights.json
