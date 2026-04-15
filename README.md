# le chess
Le chess engine.
This chess engine distills stockfish using simple linear models. The idea of this engine is to distill stockfish evaluations to form the base model.
After distilling stockfish, we make the model play against itself, and using some RL, hopefully bring it to super GM level.

The dataset for training also is custom and prepared using code, to generate random postions. As the model gets better we can feed more dynamic postions from GM/Engine games for more refinement.


# GOALS
- [X] Write the core chess engine to generate all possible moves.
- [X] Create the dataset using random moves and multithread stockfish at depth 20 (decreased to 12 bcoz of lack of compute)
- [X] Train the model
- [ ] Change dataset to contain large amounts of postional moves, hopefully makes the model develop a better intuition.
- [ ] Create specialised dataset with all famous openings and their variations and train the model on them
- [ ] Increase the depth for a smaller but more refined dataset
- [ ] Integrate the models and the engine to make a working chess bot
- [ ] use RL to further optimise the moves chosen by bot


# Extra things
- try different model archs for both models
- think of some ideas to make better datasets
- try distilling alphago and leila too
