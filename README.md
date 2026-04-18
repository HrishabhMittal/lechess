# Le Chess
Le chess engine.
This chess engine uses most of the more known chess optimisations, along with a neural network to evaluate positions.
After training it sufficiently on a dataset, we make the model play against itself, and using some RL, hopefully bring it to super GM level.

The dataset for training was custom and prepared using code (still present and can be used) to generate random postions.
Now, the model has switched to [this dataset](https://www.kaggle.com/datasets/jantyc/lichess-evaluation-dataset)

# How To Use

to play against stockfish or generate dataset, it requires stockfish to be installed at `/usr/bin/stockfish`
build the binary using
```bash
cargo build --release
```

# Demo
Here is a [game](https://lichess.org/5lp5zKLE) played by the engine at depth 10.
And this is another [game](https://lichess.org/vWJpGn5Q) which is slightly less interesting but also pretty high level.
It usually averages between 15-30 centipawn loss per move, and can see pretty interesting moves as shown in the above game.

# Todo
- [X] Write the core chess engine to generate all possible moves
- [X] Train the model to evaluate positions decently well
- [X] Integrate the models and the engine to make a working chess bot
- [ ] use RL to further optimise the moves chosen by bot
