#[derive(PartialEq, Eq, Clone, Copy)]
pub struct HintValue {
    pub new_touches: usize,
    pub delay_until_relevant: usize,
}

impl PartialOrd for HintValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HintValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.new_touches.cmp(&other.new_touches) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        other.delay_until_relevant.cmp(&self.delay_until_relevant)
    }
}
