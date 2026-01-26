//! Anomaly Detection Algorithms
//!
//! Advanced statistical methods for detecting paranormal activity patterns.

use crate::EventType;
use std::collections::VecDeque;

/// Sliding window for time-series analysis
pub struct SlidingWindow {
    data: VecDeque<f64>,
    capacity: usize,
    sum: f64,
    sum_sq: f64,
}

impl SlidingWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
            sum: 0.0,
            sum_sq: 0.0,
        }
    }
    
    /// Add value to window
    pub fn push(&mut self, value: f64) {
        if self.data.len() >= self.capacity {
            if let Some(old) = self.data.pop_front() {
                self.sum -= old;
                self.sum_sq -= old * old;
            }
        }
        
        self.data.push_back(value);
        self.sum += value;
        self.sum_sq += value * value;
    }
    
    /// Get mean
    pub fn mean(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.sum / self.data.len() as f64
    }
    
    /// Get variance
    pub fn variance(&self) -> f64 {
        if self.data.len() < 2 {
            return 0.0;
        }
        let n = self.data.len() as f64;
        (self.sum_sq - (self.sum * self.sum) / n) / (n - 1.0)
    }
    
    /// Get standard deviation
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
    
    /// Check if window is full
    pub fn is_full(&self) -> bool {
        self.data.len() >= self.capacity
    }
    
    /// Get all values
    pub fn values(&self) -> Vec<f64> {
        self.data.iter().cloned().collect()
    }
}

/// Exponential Moving Average for trend detection
pub struct ExponentialMovingAverage {
    alpha: f64,
    current: Option<f64>,
}

impl ExponentialMovingAverage {
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            current: None,
        }
    }
    
    /// Create from window size (span)
    pub fn from_span(span: usize) -> Self {
        let alpha = 2.0 / (span as f64 + 1.0);
        Self::new(alpha)
    }
    
    /// Update with new value
    pub fn update(&mut self, value: f64) -> f64 {
        let ema = match self.current {
            Some(prev) => self.alpha * value + (1.0 - self.alpha) * prev,
            None => value,
        };
        self.current = Some(ema);
        ema
    }
    
    /// Get current EMA value
    pub fn value(&self) -> Option<f64> {
        self.current
    }
    
    /// Reset EMA
    pub fn reset(&mut self) {
        self.current = None;
    }
}

/// Change Point Detection using CUSUM algorithm
pub struct ChangePointDetector {
    target_mean: f64,
    threshold: f64,
    allowance: f64,
    cusum_pos: f64,
    cusum_neg: f64,
}

impl ChangePointDetector {
    pub fn new(target_mean: f64, threshold: f64, allowance: f64) -> Self {
        Self {
            target_mean,
            threshold,
            allowance,
            cusum_pos: 0.0,
            cusum_neg: 0.0,
        }
    }
    
    /// Update with new value, returns true if change point detected
    pub fn update(&mut self, value: f64) -> bool {
        let diff = value - self.target_mean;
        
        // Update CUSUM statistics
        self.cusum_pos = (self.cusum_pos + diff - self.allowance).max(0.0);
        self.cusum_neg = (self.cusum_neg - diff - self.allowance).max(0.0);
        
        // Check for change point
        if self.cusum_pos > self.threshold || self.cusum_neg > self.threshold {
            // Reset after detection
            self.cusum_pos = 0.0;
            self.cusum_neg = 0.0;
            true
        } else {
            false
        }
    }
    
    /// Set new target mean
    pub fn set_target(&mut self, mean: f64) {
        self.target_mean = mean;
        self.cusum_pos = 0.0;
        self.cusum_neg = 0.0;
    }
}

/// Isolation Forest for multivariate anomaly detection
pub struct IsolationForest {
    trees: Vec<IsolationTree>,
    sample_size: usize,
    num_trees: usize,
}

struct IsolationTree {
    root: Option<Box<IsolationNode>>,
    height_limit: usize,
}

struct IsolationNode {
    split_feature: usize,
    split_value: f64,
    left: Option<Box<IsolationNode>>,
    right: Option<Box<IsolationNode>>,
    size: usize,
}

impl IsolationForest {
    pub fn new(num_trees: usize, sample_size: usize) -> Self {
        Self {
            trees: Vec::with_capacity(num_trees),
            sample_size,
            num_trees,
        }
    }
    
    /// Fit forest to data
    pub fn fit(&mut self, data: &[Vec<f64>]) {
        let height_limit = (self.sample_size as f64).log2().ceil() as usize;
        
        self.trees.clear();
        
        for _ in 0..self.num_trees {
            // Sample data
            let sample: Vec<&Vec<f64>> = data.iter()
                .take(self.sample_size)
                .collect();
            
            // Build tree
            let root = self.build_tree(&sample, 0, height_limit);
            self.trees.push(IsolationTree {
                root: Some(root),
                height_limit,
            });
        }
    }
    
    fn build_tree(&self, data: &[&Vec<f64>], depth: usize, height_limit: usize) -> Box<IsolationNode> {
        if depth >= height_limit || data.len() <= 1 {
            return Box::new(IsolationNode {
                split_feature: 0,
                split_value: 0.0,
                left: None,
                right: None,
                size: data.len(),
            });
        }
        
        let num_features = data.first().map(|v| v.len()).unwrap_or(0);
        if num_features == 0 {
            return Box::new(IsolationNode {
                split_feature: 0,
                split_value: 0.0,
                left: None,
                right: None,
                size: data.len(),
            });
        }
        
        // Random feature selection
        let split_feature = simple_random(num_features);
        
        // Find min/max for selected feature
        let (min_val, max_val) = data.iter()
            .filter_map(|v| v.get(split_feature))
            .fold((f64::MAX, f64::MIN), |(min, max), &v| {
                (min.min(v), max.max(v))
            });
        
        if (max_val - min_val).abs() < f64::EPSILON {
            return Box::new(IsolationNode {
                split_feature,
                split_value: min_val,
                left: None,
                right: None,
                size: data.len(),
            });
        }
        
        // Random split value
        let split_value = min_val + simple_random_f64() * (max_val - min_val);
        
        // Partition data
        let (left_data, right_data): (Vec<_>, Vec<_>) = data.iter()
            .partition(|v| v.get(split_feature).map(|&x| x < split_value).unwrap_or(false));
        
        Box::new(IsolationNode {
            split_feature,
            split_value,
            left: Some(self.build_tree(&left_data, depth + 1, height_limit)),
            right: Some(self.build_tree(&right_data, depth + 1, height_limit)),
            size: data.len(),
        })
    }
    
    /// Calculate anomaly score for a point (0-1, higher = more anomalous)
    pub fn score(&self, point: &[f64]) -> f64 {
        if self.trees.is_empty() {
            return 0.5;
        }
        
        let avg_path_length: f64 = self.trees.iter()
            .map(|tree| self.path_length(point, &tree.root, 0) as f64)
            .sum::<f64>() / self.trees.len() as f64;
        
        // Normalize using expected path length
        let c = self.expected_path_length(self.sample_size);
        
        // Anomaly score
        2.0_f64.powf(-avg_path_length / c)
    }
    
    fn path_length(&self, point: &[f64], node: &Option<Box<IsolationNode>>, depth: usize) -> usize {
        match node {
            None => depth,
            Some(n) => {
                if n.left.is_none() && n.right.is_none() {
                    return depth + self.expected_path_length(n.size) as usize;
                }
                
                let value = point.get(n.split_feature).copied().unwrap_or(0.0);
                
                if value < n.split_value {
                    self.path_length(point, &n.left, depth + 1)
                } else {
                    self.path_length(point, &n.right, depth + 1)
                }
            }
        }
    }
    
    fn expected_path_length(&self, n: usize) -> f64 {
        if n <= 1 {
            return 0.0;
        }
        2.0 * (harmonic_number(n - 1) - (n as f64 - 1.0) / n as f64)
    }
}

/// Pattern matcher for recurring anomalies
pub struct PatternMatcher {
    patterns: Vec<Pattern>,
    window_size: usize,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub signature: Vec<f64>,
    pub tolerance: f64,
    pub event_type: EventType,
}

impl PatternMatcher {
    pub fn new(window_size: usize) -> Self {
        Self {
            patterns: Vec::new(),
            window_size,
        }
    }
    
    /// Add pattern to match against
    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
    }
    
    /// Match window against known patterns
    pub fn match_patterns(&self, window: &[f64]) -> Vec<(Pattern, f64)> {
        let mut matches = Vec::new();
        
        for pattern in &self.patterns {
            let similarity = self.calculate_similarity(window, &pattern.signature);
            
            if similarity >= pattern.tolerance {
                matches.push((pattern.clone(), similarity));
            }
        }
        
        // Sort by similarity (highest first)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        matches
    }
    
    fn calculate_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        
        // Normalized cross-correlation
        let mean_a: f64 = a.iter().sum::<f64>() / a.len() as f64;
        let mean_b: f64 = b.iter().sum::<f64>() / b.len() as f64;
        
        let mut num = 0.0;
        let mut denom_a = 0.0;
        let mut denom_b = 0.0;
        
        let len = a.len().min(b.len());
        
        for i in 0..len {
            let diff_a = a[i] - mean_a;
            let diff_b = b[i] - mean_b;
            
            num += diff_a * diff_b;
            denom_a += diff_a * diff_a;
            denom_b += diff_b * diff_b;
        }
        
        let denom = (denom_a * denom_b).sqrt();
        
        if denom < f64::EPSILON {
            return 0.0;
        }
        
        (num / denom + 1.0) / 2.0  // Normalize to 0-1
    }
    
    /// Learn pattern from labeled data
    pub fn learn_pattern(&mut self, name: &str, samples: &[Vec<f64>], event_type: EventType) {
        if samples.is_empty() {
            return;
        }
        
        let len = samples[0].len();
        let mut signature = vec![0.0; len];
        
        // Average all samples
        for sample in samples {
            for (i, &val) in sample.iter().enumerate() {
                if i < len {
                    signature[i] += val;
                }
            }
        }
        
        for val in &mut signature {
            *val /= samples.len() as f64;
        }
        
        self.add_pattern(Pattern {
            name: name.to_string(),
            signature,
            tolerance: 0.7,
            event_type,
        });
    }
}

// Helper functions

fn harmonic_number(n: usize) -> f64 {
    (1..=n).map(|i| 1.0 / i as f64).sum()
}

fn simple_random(max: usize) -> usize {
    static mut SEED: u64 = 42;
    unsafe {
        SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1);
        (SEED >> 33) as usize % max
    }
}

fn simple_random_f64() -> f64 {
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1);
        (SEED >> 11) as f64 / (1u64 << 53) as f64
    }
}
