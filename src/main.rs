mod board;
mod eval;
mod gen_dataset;

use crate::gen_dataset::gen_dataset;
fn main() {
    gen_dataset();
}
