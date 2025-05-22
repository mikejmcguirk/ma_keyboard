pub struct IdSpawner {
    next_id: usize,
}

impl IdSpawner {
    pub fn new() -> Self {
        return Self { next_id: 0 };
    }

    // PERF: I want to return 0 as the first id but maybe this is an extravagance
    pub fn get(&mut self) -> usize {
        let to_return: usize = self.next_id;
        self.next_id += 1;

        return to_return;
    }
}
