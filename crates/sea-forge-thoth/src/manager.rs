use sea_forge_core::types::ManagerJudgment;

/// Deterministic case-state facts the manager loop judges against (§7.6,
/// §9.5). Built by the caller from `next_case_actions` + case state — this
/// module stays IO-free, mirroring `engine.rs`'s `SnapshotView` pattern.
pub struct ManagerView {
    /// The case has already reached a completed state.
    pub case_completed: bool,
    /// An unresolved approval, missing required authority, or terminal
    /// dependency block exists (`TerminateCase`/`ParkHumanTask` actions, or
    /// `CaseState::AwaitingApproval`).
    pub blocked: bool,
    /// At least one item is active or enabled right now (`Enable`/
    /// `Activate`/`AchieveMilestone` actions pending).
    pub has_ready_work: bool,
    /// A new settlement was recorded since the prior iteration's snapshot.
    pub new_settlement_progress: bool,
}

/// Classify a case deterministically per §9.5's rule table. Never invents
/// free-form judgment — purely a function of the observed facts.
pub fn judge(view: &ManagerView) -> ManagerJudgment {
    if view.case_completed {
        ManagerJudgment::Satisfied
    } else if view.blocked {
        ManagerJudgment::Blocked
    } else if view.has_ready_work || view.new_settlement_progress {
        ManagerJudgment::Progressing
    } else {
        ManagerJudgment::Stalled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_is_satisfied_regardless_of_other_facts() {
        let view = ManagerView {
            case_completed: true,
            blocked: true,
            has_ready_work: true,
            new_settlement_progress: true,
        };
        assert_eq!(judge(&view), ManagerJudgment::Satisfied);
    }

    #[test]
    fn blocked_beats_ready_work() {
        let view = ManagerView {
            case_completed: false,
            blocked: true,
            has_ready_work: true,
            new_settlement_progress: false,
        };
        assert_eq!(judge(&view), ManagerJudgment::Blocked);
    }

    #[test]
    fn ready_work_is_progressing() {
        let view = ManagerView {
            case_completed: false,
            blocked: false,
            has_ready_work: true,
            new_settlement_progress: false,
        };
        assert_eq!(judge(&view), ManagerJudgment::Progressing);
    }

    #[test]
    fn no_work_no_progress_is_stalled() {
        let view = ManagerView {
            case_completed: false,
            blocked: false,
            has_ready_work: false,
            new_settlement_progress: false,
        };
        assert_eq!(judge(&view), ManagerJudgment::Stalled);
    }
}
