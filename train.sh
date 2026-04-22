#!/bin/bash

GAMES_PER_GEN=500
DEPTH=4
DATA_FILE="self_play_data.txt"
WEIGHTS_FILE="models/weights.msgpack"
GENERATION=1

cargo build --release

while true; do
    
    echo gen $GENERATION

    ./target/release/duchess \
        --gen-dataset \
        --data-size $GAMES_PER_GEN \
        --dataset-file $DATA_FILE \
        --weights-file $WEIGHTS_FILE \
        --depth $DEPTH
        
    if [ $? -ne 0 ]; then
        echo "gen failed. exiting."
        exit 1
    fi

    echo training on data
    python models/train.py

    if [ $? -ne 0 ]; then
        echo "training failed. exiting."
        exit 1
    fi

    python models/export.py

    tail -n 500000 $DATA_FILE > temp_data.txt && mv temp_data.txt $DATA_FILE
    ((GENERATION++))
done
