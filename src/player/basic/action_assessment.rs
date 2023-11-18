#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ActionAssessment {
    pub new_touches: usize,
    pub delay_until_relevant: usize,
    pub is_unconventional: bool,
    pub action_type: ActionType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionType {
    Play,
    Discard,
    Hint,
}

impl ActionAssessment {
    pub(crate) fn unconvectional() -> Self {
        Self {
            is_unconventional: true,
            new_touches: 100,
            delay_until_relevant: 0,
            action_type: ActionType::Hint,
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

        match (self.action_type == ActionType::Play).cmp(&(other.action_type == ActionType::Play)) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match (self.action_type == ActionType::Hint).cmp(&(other.action_type == ActionType::Hint)) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        assert_eq!(self.action_type, other.action_type);

        match self.new_touches.cmp(&other.new_touches) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        other.delay_until_relevant.cmp(&self.delay_until_relevant)
    }
}
