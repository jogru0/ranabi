#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ActionAssessment {
    pub new_touches: usize,
    pub delay_until_relevant: usize,
    pub is_unconventional: bool,
    pub plays_a_card_right_now: bool,
}
impl ActionAssessment {
    pub(crate) fn unconvectional() -> Self {
        Self {
            is_unconventional: true,
            new_touches: 100,
            delay_until_relevant: 0,
            plays_a_card_right_now: true,
        }
    }
}

impl PartialOrd for ActionAssessment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActionAssessment {
    //Want better/truer => put self first.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.is_unconventional.cmp(&self.is_unconventional) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match self
            .plays_a_card_right_now
            .cmp(&self.plays_a_card_right_now)
        {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match self.new_touches.cmp(&other.new_touches) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        other.delay_until_relevant.cmp(&self.delay_until_relevant)
    }
}
