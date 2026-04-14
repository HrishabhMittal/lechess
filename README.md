# le chess
Le chess engine.
This chess engine distills stockfish using simple linear models. The idea of this engine is to use a smaller model to order move search in alpha-beta pruning, then have a slightly bigger model run evaluation on the max depth layer.
The first model is specialised in ordering all possible moves, while the second heavier model provides a static evaluation of the position.

After distilling stockfish, we make the model play against itself, and using some RL, hopefully bring it to super GM level.

The dataset for training also is custom and prepared using code, to generate random postions. As the model gets better we can feed more dynamic postions from GM/Engine games for more refinement.


# GOALS
- [X] Write the core chess engine to generate all possible moves.
- [ ] Create the dataset using random moves and multithread stockfish at depth 20
- [ ] Train the models
- [ ] Create specialised dataset with all famous openings and their variations and train the model on them
- [ ] Increase the depth for a smaller but more refined dataset
- [ ] Integrate the models and the engine to make a working chess bot
- [ ] use RL to further optimise the moves chosen by bot


# Extra things
- try different model archs for both models
- think of some ideas to make better datasets
- try distilling alphago and leila too
