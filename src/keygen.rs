pub struct HexKeyGenerator {
    current: [u8; 32],
    end: [u8; 32],
}

impl HexKeyGenerator {
    pub fn new(start_hex: &str, end_hex: &str) -> Self {
        let mut current = [0u8; 32];
        let mut end = [0u8; 32];
        hex::decode_to_slice(start_hex, &mut current).expect("Invalid start hex");
        hex::decode_to_slice(end_hex, &mut end).expect("Invalid end hex");
        Self { current, end }
    }

    pub fn next_batch(&mut self, batch_size: usize) -> Vec<[u8; 32]> {
        let mut batch = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            batch.push(self.current);

            if self.current == self.end {
                break;
            }

            Self::increment(&mut self.current);
        }
        batch
    }


    pub fn last_key(&self) -> String {
        hex::encode(self.current)
    }

    fn increment(key: &mut [u8; 32]) {
        for i in (0..32).rev() {
            if key[i] < 0xFF {
                key[i] += 1;
                break;
            } else {
                key[i] = 0;
            }
        }
    }
}