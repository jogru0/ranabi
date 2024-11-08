#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ActionAssessment {
    pub new_touches: usize,
    pub delay_until_relevant: usize,
    is_unconventional: bool,
    pub action_type: ActionType,
    pub sure_influence_on_clue_count: i8,
    pub last_resort: bool,
    pub next_player_might_be_locked_with_no_clue: bool,
    tempo: usize,
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
            sure_influence_on_clue_count: 9,
            last_resort: false,
            next_player_might_be_locked_with_no_clue: false,
            tempo: 1000,
        }
    }

    pub fn is_unconventional(&self) -> &bool {
        &self.is_unconventional
    }

    pub fn new(
        new_touches: usize,
        delay_until_relevant: usize,
        action_type: ActionType,
        sure_influence_on_clue_count: i8,
        last_resort: bool,
        tempo: usize,
    ) -> Self {
        Self {
            new_touches,
            delay_until_relevant,
            is_unconventional: false,
            action_type,
            sure_influence_on_clue_count,
            last_resort,
            next_player_might_be_locked_with_no_clue: false,
            tempo,
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

        match other.last_resort.cmp(&self.last_resort) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match other
            .next_player_might_be_locked_with_no_clue
            .cmp(&self.next_player_might_be_locked_with_no_clue)
        {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match (self.action_type == ActionType::Play).cmp(&(other.action_type == ActionType::Play)) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match self.new_touches.cmp(&other.new_touches) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // match self.tempo.cmp(&other.tempo) {
        //     core::cmp::Ordering::Equal => {}
        //     ord => return ord,
        // }

        match self
            .sure_influence_on_clue_count
            .cmp(&other.sure_influence_on_clue_count)
        {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        other.delay_until_relevant.cmp(&self.delay_until_relevant)
    }
}
