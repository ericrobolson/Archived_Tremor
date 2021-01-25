pub struct CircleBuffer<T>
where
    T: Copy,
{
    size: usize,
    data: Vec<T>,
}

impl<T> CircleBuffer<T>
where
    T: Copy,
{
    pub fn new(size: usize, default: T) -> Self {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(default);
        }

        Self {
            size: size,
            data: data,
        }
    }

    pub fn insert(&mut self, i: usize, item: T) {
        self.data[i % self.size] = item;
    }

    pub fn item(&self, i: usize) -> &T {
        &self.data[i % self.size]
    }
}
