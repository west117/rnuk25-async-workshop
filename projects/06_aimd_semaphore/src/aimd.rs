#[derive(Debug)]
pub struct Aimd {
    limit: u64,
    config: AimdConfig,
}

#[derive(Debug)]
pub struct AimdConfig {
    pub min: u64,
    pub max: u64,
    pub inc: u64,
    pub dec: f64,
}

impl Aimd {
    pub fn new(config: AimdConfig) -> Self {
        Self {
            limit: config.max,
            config,
        }
    }

    pub fn success(&mut self) {
        self.limit += self.config.inc;
        self.limit = self.limit.clamp(self.config.min, self.config.max);
    }

    pub fn failure(&mut self) {
        self.limit = ((self.limit as f64) * self.config.dec).round() as u64;
        self.limit = self.limit.clamp(self.config.min, self.config.max);
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }
}
