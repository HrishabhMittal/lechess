use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
pub struct ChessNetWeights {
    fc1_weight: Vec<Vec<f32>>,
    fc1_bias: Vec<f32>,
    fc2_weight: Vec<Vec<f32>>,
    fc2_bias: Vec<f32>,
    fc3_weight: Vec<Vec<f32>>,
    fc3_bias: Vec<f32>,
    fc4_weight: Vec<Vec<f32>>,
    fc4_bias: Vec<f32>,
}

pub struct NeuralNet {
    weights: ChessNetWeights,
}

impl NeuralNet {
    pub fn load(path: &str) -> Self {
        let file = File::open(path).expect("couldnt open weights.json");
        let reader = BufReader::new(file);
        let weights: ChessNetWeights = serde_json::from_reader(reader).expect("couldnt parse JSON");

        NeuralNet { weights }
    }
    pub fn evaluate(&self, features: &[f32; 772]) -> f32 {
        let mut l1 = [0.0; 256];
        for i in 0..256 {
            let mut sum = self.weights.fc1_bias[i];

            for j in 0..772 {
                if features[j] > 0.0 {
                    sum += self.weights.fc1_weight[i][j];
                }
            }

            l1[i] = sum.clamp(0.0, 1.0);
        }

        let mut l2 = [0.0; 32];
        for i in 0..32 {
            let mut sum = self.weights.fc2_bias[i];
            for j in 0..256 {
                sum += l1[j] * self.weights.fc2_weight[i][j];
            }

            l2[i] = sum.clamp(0.0, 1.0);
        }

        let mut l3 = [0.0; 32];
        for i in 0..32 {
            let mut sum = self.weights.fc3_bias[i];
            for j in 0..32 {
                sum += l2[j] * self.weights.fc3_weight[i][j];
            }

            l3[i] = sum.clamp(0.0, 1.0);
        }

        let mut output = self.weights.fc4_bias[0];
        for i in 0..32 {
            output += l3[i] * self.weights.fc4_weight[0][i];
        }

        let sigmoid = 1.0 / (1.0 + (-output).exp());
        let clamped = sigmoid.clamp(0.0001, 0.9999);
        let centipawns = -400.0 * ((1.0 / clamped) - 1.0).ln();

        centipawns
    }
}
