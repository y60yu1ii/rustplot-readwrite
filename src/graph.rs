use rand::Rng;
use std::f64::consts::PI;

pub enum GraphType {
    Triangle,
    SinWave,
}

pub struct Graph {
    pub data: Vec<(f64, f64)>,
    pub graph_type: GraphType,
    increasing: bool,
    i: f64,
    max_value: f64,
    frequency: f64, // ✅ Sin 波的隨機頻率
}

impl Graph {
    pub fn new(graph_type: GraphType) -> Self {
        let mut rng = rand::rng();
        Self {
            data: Vec::new(),
            graph_type,
            increasing: true,
            i: 0.0,
            max_value: rng.random_range(100.0..255.0), // ✅ 產生隨機最大值
            frequency: rng.random_range(0.5..2.0),     // ✅ 產生隨機 Sin 波頻率
        }
    }

    pub fn update(&mut self, elapsed: f64) {
        match self.graph_type {
            GraphType::Triangle => {
                if self.increasing {
                    self.i += 1.0;
                    if self.i >= self.max_value {
                        self.increasing = false;
                    }
                } else {
                    self.i -= 1.0;
                    if self.i <= 0.0 {
                        self.increasing = true;
                        let mut rng = rand::rng();
                        self.max_value = rng.random_range(100.0..255.0); // ✅ 重新產生隨機最大值
                    }
                }
            }
            GraphType::SinWave => {
                self.i = (elapsed * self.frequency * 2.0 * PI).sin() * self.max_value;
                // ✅ Sin 波
            }
        }

        // 限制最多顯示最近 10 秒的數據
        self.data.push((elapsed, self.i));
        self.data.retain(|&(x, _)| elapsed - x < 10.0);
    }
}
