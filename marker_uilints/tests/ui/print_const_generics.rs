use std::fmt::Debug;

// FIXME: Make sure this cluster duck works
pub struct ArrayPair<T, const N: usize> {
    left: [T; N],
    right: [T; N],
}

impl<T: Debug, const N: usize> ArrayPair<T, N> {
    fn print(&self) {
        println!("left: {:?}", self.left);
        println!("right: {:?}", self.right);
    }
}

fn main() {
    let _ty = ArrayPair {
        left: [1, 2, 3],
        right: [4, 5, 6],
    };
    // let arr = ArrayPair {
    //     left: [1, 2, 3],
    //     right: [4, 5, 6],
    // };
    // arr.print();
}
