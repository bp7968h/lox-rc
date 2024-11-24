pub struct Scanner {
    line: usize,
    start: usize,
    current: usize,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            line: 1,
            start: 0,
            current: 0,
        }
    }
}