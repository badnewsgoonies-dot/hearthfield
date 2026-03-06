use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::components::NpcRole;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct TaskId(pub u64);

#[allow(dead_code)]
impl TaskId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[allow(dead_code)]
impl From<u64> for TaskId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum TaskKind {
    #[default]
    DataEntry,
    Filing,
    EmailTriage,
    PermitReview,
    ClientCall,
    MeetingPrep,
    ReportWriting,
    BudgetReview,
}

#[allow(dead_code)]
impl TaskKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DataEntry => "Data Entry",
            Self::Filing => "Filing",
            Self::EmailTriage => "Email Triage",
            Self::PermitReview => "Permit Review",
            Self::ClientCall => "Client Call",
            Self::MeetingPrep => "Meeting Prep",
            Self::ReportWriting => "Report Writing",
            Self::BudgetReview => "Budget Review",
        }
    }

    pub fn base_duration_secs(&self) -> u32 {
        match self {
            Self::DataEntry => 60,
            Self::Filing => 75,
            Self::EmailTriage => 90,
            Self::PermitReview => 150,
            Self::ReportWriting => 180,
            Self::MeetingPrep => 120,
            Self::ClientCall => 90,
            Self::BudgetReview => 240,
        }
    }

    pub fn base_xp(&self) -> u32 {
        match self {
            Self::DataEntry => 8,
            Self::Filing => 10,
            Self::EmailTriage => 12,
            Self::PermitReview => 18,
            Self::ReportWriting => 15,
            Self::MeetingPrep => 10,
            Self::ClientCall => 20,
            Self::BudgetReview => 25,
        }
    }

    pub fn stress_factor(&self) -> f32 {
        match self {
            Self::DataEntry => 0.1,
            Self::Filing => 0.15,
            Self::EmailTriage => 0.25,
            Self::PermitReview => 0.35,
            Self::ReportWriting => 0.3,
            Self::MeetingPrep => 0.2,
            Self::ClientCall => 0.5,
            Self::BudgetReview => 0.4,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::DataEntry => "Enter spreadsheet figures for quarterly report",
            Self::Filing => "Sort and file incoming documents to correct departments",
            Self::EmailTriage => "Read, prioritize, and route the email backlog",
            Self::PermitReview => "Check permit applications for compliance",
            Self::ClientCall => "Call a client and resolve open questions",
            Self::MeetingPrep => "Prepare agenda and handouts for upcoming meeting",
            Self::ReportWriting => "Draft the weekly status report for management",
            Self::BudgetReview => "Reconcile expense receipts against budget lines",
        }
    }

    pub fn label(&self) -> &'static str {
        self.display_name()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

#[allow(dead_code)]
impl TaskPriority {
    pub const fn rank(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    pub fn normalized(self) -> f32 {
        (self.rank() as f32 - 1.0) / 3.0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OfficeTask {
    pub id: TaskId,
    pub kind: TaskKind,
    pub priority: TaskPriority,
    pub required_focus: i32,
    pub stress_impact: i32,
    pub reward_money: i32,
    pub reward_reputation: i32,
    pub deadline_minute: u16,
    pub progress: f32,
}

#[allow(dead_code)]
impl Default for OfficeTask {
    fn default() -> Self {
        Self {
            id: TaskId::default(),
            kind: TaskKind::default(),
            priority: TaskPriority::default(),
            required_focus: 0,
            stress_impact: 0,
            reward_money: 0,
            reward_reputation: 0,
            deadline_minute: 17 * 60,
            progress: 0.0,
        }
    }
}

#[allow(dead_code)]
impl OfficeTask {
    pub fn normalize(&mut self) {
        self.required_focus = self.required_focus.max(0);
        self.progress = self.progress.clamp(0.0, 1.0);
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 0.999
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum InterruptionKind {
    #[default]
    ManagerRequest,
    EmergencyMeeting,
    SystemOutage,
    CoworkerHelp,
    PrinterJam,
    FireDrill,
    BossVisit,
    FreeLunch,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum ChoiceId {
    #[default]
    A,
    B,
    C,
}

#[allow(dead_code)]
#[derive(Resource, Debug, Clone, Default)]
pub struct TaskBoard {
    pub active: Vec<OfficeTask>,
    pub completed_today: Vec<TaskId>,
    pub failed_today: Vec<TaskId>,
}

#[allow(dead_code)]
impl TaskBoard {
    pub fn has_active_task(&self, task_id: TaskId) -> bool {
        self.active.iter().any(|task| task.id == task_id)
    }

    pub fn is_completed(&self, task_id: TaskId) -> bool {
        self.completed_today.contains(&task_id)
    }

    pub fn is_failed(&self, task_id: TaskId) -> bool {
        self.failed_today.contains(&task_id)
    }

    pub fn try_add_task(&mut self, task: OfficeTask) -> bool {
        if self.has_active_task(task.id) {
            return false;
        }

        self.active.push(task.normalized());
        true
    }

    pub fn complete_task(&mut self, task_id: TaskId) -> bool {
        if self.is_completed(task_id) || self.is_failed(task_id) {
            return false;
        }

        let Some(index) = self.active.iter().position(|task| task.id == task_id) else {
            return false;
        };

        self.active.remove(index);
        self.completed_today.push(task_id);
        self.normalize();
        true
    }

    pub fn progress_task(&mut self, task_id: TaskId, delta: f32) -> Option<f32> {
        if delta <= 0.0 || self.is_completed(task_id) || self.is_failed(task_id) {
            return None;
        }

        let task = self.active.iter_mut().find(|task| task.id == task_id)?;
        let before = task.progress;
        task.progress = (task.progress + delta).clamp(0.0, 1.0);
        let applied = task.progress - before;
        if applied <= 0.0 {
            return None;
        }

        task.normalize();
        Some(applied)
    }

    pub fn fail_task(&mut self, task_id: TaskId) -> bool {
        if self.is_completed(task_id) || self.is_failed(task_id) {
            return false;
        }

        let Some(index) = self.active.iter().position(|task| task.id == task_id) else {
            return false;
        };

        self.active.remove(index);
        self.failed_today.push(task_id);
        self.normalize();
        true
    }

    pub fn active_task_ids(&self) -> Vec<TaskId> {
        self.active.iter().map(|task| task.id).collect()
    }

    pub fn normalize(&mut self) {
        for task in &mut self.active {
            task.normalize();
        }

        let mut seen_active = HashSet::new();
        self.active.retain(|task| seen_active.insert(task.id));

        let mut seen_completed = HashSet::new();
        self.completed_today
            .retain(|task_id| seen_completed.insert(*task_id));

        let mut seen_failed = HashSet::new();
        self.failed_today
            .retain(|task_id| !seen_completed.contains(task_id) && seen_failed.insert(*task_id));
    }
}

#[allow(dead_code)]
#[derive(Resource, Debug, Clone)]
pub struct OfficeRunConfig {
    pub seed: u64,
    pub max_tasks_per_day: u8,
    pub interruption_chance_per_hour: f32,
}

#[allow(dead_code)]
impl Default for OfficeRunConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            max_tasks_per_day: 6,
            interruption_chance_per_hour: 0.2,
        }
    }
}

#[allow(dead_code)]
impl OfficeRunConfig {
    pub fn normalize(&mut self) {
        self.max_tasks_per_day = self.max_tasks_per_day.max(1);
        self.interruption_chance_per_hour = self.interruption_chance_per_hour.clamp(0.0, 1.0);
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

#[allow(dead_code)]
#[derive(Resource, Debug, Clone, Default)]
pub struct DayOutcome {
    pub salary_earned: i32,
    pub reputation_delta: i32,
    pub stress_delta: i32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
}

#[allow(dead_code)]
impl DayOutcome {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[allow(dead_code)]
#[derive(Resource, Debug, Clone)]
pub struct WorkerStats {
    pub energy: i32,
    pub stress: i32,
    pub focus: i32,
    pub money: i32,
    pub reputation: i32,
}

#[allow(dead_code)]
impl Default for WorkerStats {
    fn default() -> Self {
        Self {
            energy: 100,
            stress: 18,
            focus: 76,
            money: 0,
            reputation: 0,
        }
    }
}

#[allow(dead_code)]
impl WorkerStats {
    pub fn normalize(&mut self) {
        self.energy = self.energy.clamp(0, 100);
        self.stress = self.stress.clamp(0, 100);
        self.focus = self.focus.clamp(0, 100);
        self.reputation = self.reputation.clamp(-100, 100);
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

#[derive(Resource, Debug)]
pub struct OfficeRules {
    pub max_energy: i32,
    pub max_stress: i32,
    pub max_focus: i32,
    pub max_reputation: i32,
    pub process_energy_cost: i32,
    pub process_minutes: u32,
    pub coffee_restore: i32,
    pub coffee_minutes: u32,
    pub wait_minutes: u32,
    pub interruption_minutes: u32,
    pub interruption_stress_increase: i32,
    pub interruption_focus_loss: i32,
    pub calm_focus_restore: i32,
    pub calm_stress_relief: i32,
    pub panic_stress_increase: i32,
    pub panic_focus_loss: i32,
    pub manager_checkin_minutes: u32,
    pub manager_checkin_stress_increase: i32,
    pub manager_checkin_reputation_gain: i32,
    pub coworker_help_minutes: u32,
    pub coworker_help_focus_cost: i32,
    pub coworker_help_stress_relief: i32,
    pub coworker_help_reputation_gain: i32,
    pub starting_stress: i32,
    pub starting_focus: i32,
    pub starting_reputation: i32,
    pub day_start_minute: u32,
    pub day_end_minute: u32,
    pub starting_inbox_items: u32,
}

impl Default for OfficeRules {
    fn default() -> Self {
        Self {
            max_energy: 100,
            max_stress: 100,
            max_focus: 100,
            max_reputation: 100,
            process_energy_cost: 12,
            process_minutes: 15,
            coffee_restore: 25,
            coffee_minutes: 20,
            wait_minutes: 10,
            interruption_minutes: 12,
            interruption_stress_increase: 14,
            interruption_focus_loss: 10,
            calm_focus_restore: 12,
            calm_stress_relief: 14,
            panic_stress_increase: 11,
            panic_focus_loss: 7,
            manager_checkin_minutes: 8,
            manager_checkin_stress_increase: 10,
            manager_checkin_reputation_gain: 2,
            coworker_help_minutes: 6,
            coworker_help_focus_cost: 6,
            coworker_help_stress_relief: 4,
            coworker_help_reputation_gain: 3,
            starting_stress: 18,
            starting_focus: 76,
            starting_reputation: 0,
            day_start_minute: 9 * 60,
            day_end_minute: 17 * 60,
            starting_inbox_items: 12,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct InboxState {
    pub remaining_items: u32,
}

impl Default for InboxState {
    fn default() -> Self {
        Self {
            remaining_items: 18,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct DayClock {
    pub day_number: u32,
    pub current_minute: u32,
    pub ended: bool,
}

impl Default for DayClock {
    fn default() -> Self {
        Self {
            day_number: 1,
            current_minute: 9 * 60,
            ended: false,
        }
    }
}

impl DayClock {
    pub fn advance(&mut self, minutes: u32) {
        self.current_minute = self.current_minute.saturating_add(minutes);
    }
}

#[derive(Resource, Debug)]
pub struct PlayerMindState {
    pub stress: i32,
    pub focus: i32,
    pub pending_interruptions: u32,
}

impl Default for PlayerMindState {
    fn default() -> Self {
        Self {
            stress: 18,
            focus: 76,
            pending_interruptions: 0,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct PlayerCareerState {
    pub reputation: i32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CoworkerRole {
    Manager,
    Clerk,
    Analyst,
    Coordinator,
    Intern,
    Specialist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum CoworkerPersonality {
    #[default]
    Neutral,
    Ambitious,    // competitive, seeks promotions
    Nurturing,    // helpful, mentoring
    Skeptical,    // suspicious, needs proof
    Enthusiastic, // energetic, sometimes overwhelming
    Reserved,     // quiet, observant
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ManagerArcStage {
    #[default]
    Stranger,     // trust < 10
    Acquaintance, // trust 10-29
    Mentor,       // trust 30-59, affinity >= 20
    Evaluator,    // trust 30-59, affinity < 20
    Ally,         // trust >= 60, affinity >= 40
    Antagonist,   // trust >= 60, affinity < 0
}

impl CoworkerRole {
    pub const fn npc_role(self) -> NpcRole {
        match self {
            CoworkerRole::Manager => NpcRole::Manager,
            CoworkerRole::Clerk => NpcRole::Clerk,
            CoworkerRole::Analyst => NpcRole::Analyst,
            CoworkerRole::Coordinator => NpcRole::Coordinator,
            CoworkerRole::Intern => NpcRole::Intern,
            CoworkerRole::Specialist => NpcRole::Analyst, // Specialists appear as Analysts in NPC representation
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct CoworkerDialogue {
    pub lines: HashMap<NpcRole, Vec<String>>,
}

impl CoworkerDialogue {
    pub fn line_for_role(&self, role: CoworkerRole, seed: u64, cursor: u32) -> Option<&str> {
        let lines = self.lines.get(&role.npc_role())?;
        if lines.is_empty() {
            return None;
        }
        let index = (seed.wrapping_add(cursor as u64 * 11) % lines.len() as u64) as usize;
        lines.get(index).map(String::as_str)
    }

    /// Returns a dialogue line that varies based on personality.
    /// Ambitious and Enthusiastic get energetic lines, Skeptical gets cautious lines, etc.
    #[allow(dead_code)]
    pub fn line_for_personality(
        &self,
        role: CoworkerRole,
        personality: CoworkerPersonality,
        seed: u64,
    ) -> String {
        let base_lines = self.lines.get(&role.npc_role());
        let base = base_lines
            .and_then(|lines| {
                if lines.is_empty() {
                    None
                } else {
                    let index = (seed % lines.len() as u64) as usize;
                    lines.get(index)
                }
            })
            .map(String::as_str)
            .unwrap_or("...");

        // Personality-specific prefixes/suffixes that color the dialogue
        match personality {
            CoworkerPersonality::Ambitious => {
                let variants = [
                    format!("{} We should aim higher.", base),
                    format!("Let's be strategic: {}", base),
                    format!("{} This could be our breakthrough.", base),
                ];
                variants[(seed % 3) as usize].clone()
            }
            CoworkerPersonality::Nurturing => {
                let variants = [
                    format!("{} Let me know if you need help.", base),
                    format!("Don't worry, you've got this. {}", base),
                    format!("{} I'm here if you want to talk it through.", base),
                ];
                variants[(seed % 3) as usize].clone()
            }
            CoworkerPersonality::Skeptical => {
                let variants = [
                    format!("{} ...but let's verify that.", base),
                    format!("Hmm. {} We'll see.", base),
                    format!("{} I'll believe it when I see the data.", base),
                ];
                variants[(seed % 3) as usize].clone()
            }
            CoworkerPersonality::Enthusiastic => {
                let variants = [
                    format!("Oh! {} This is exciting!", base),
                    format!("{} I love this!", base),
                    format!("Amazing! {}", base),
                ];
                variants[(seed % 3) as usize].clone()
            }
            CoworkerPersonality::Reserved => {
                let variants = [
                    format!("...{}", base),
                    format!("{} *nods quietly*", base),
                    format!("Mm. {}", base),
                ];
                variants[(seed % 3) as usize].clone()
            }
            CoworkerPersonality::Neutral => base.to_string(),
        }
    }
}

impl Default for CoworkerDialogue {
    fn default() -> Self {
        let mut lines = HashMap::new();
        lines.insert(
            NpcRole::Manager,
            vec![
                "Let's tighten the deadline and keep our KPIs green.".to_string(),
                "Quick sync: how are we tracking against this week's targets?".to_string(),
                "Love the momentum, now let's turn that into team synergy.".to_string(),
            ],
        );
        lines.insert(
            NpcRole::Intern,
            vec![
                "Is it normal to be this excited about the office coffee machine?".to_string(),
                "If I color-code my notes, does that count as process improvement?".to_string(),
                "Wait, what's a KPI again? I wrote it down as 'keep people inspired'.".to_string(),
            ],
        );
        lines.insert(
            NpcRole::Clerk,
            vec![
                "Inbox zero by lunch is still alive, somehow.".to_string(),
                "I alphabetized the forms and now I feel unstoppable.".to_string(),
                "If the printer jams again, I'm taking a tactical snack break.".to_string(),
            ],
        );
        lines.insert(
            NpcRole::Analyst,
            vec![
                "The trend line says we're improving, even if slowly.".to_string(),
                "I made a dashboard for this dashboard so we can monitor the first dashboard."
                    .to_string(),
                "Give me ten minutes and I'll have a chart for that feeling.".to_string(),
            ],
        );
        lines.insert(
            NpcRole::Coordinator,
            vec![
                "I've booked the room, moved the meeting, and fixed the calendar collision."
                    .to_string(),
                "If everyone replies-all one more time, I'm muting this thread.".to_string(),
                "Let's keep this simple: clear owner, clear deadline, clear handoff.".to_string(),
            ],
        );
        Self { lines }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoworkerProfile {
    pub id: u8,
    pub codename: String,
    pub role: CoworkerRole,
    pub affinity: i32,
    pub trust: i32,
    pub personality: CoworkerPersonality,
    pub backstory: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SocialGraphState {
    pub profiles: Vec<CoworkerProfile>,
    pub scenario_cursor: u32,
    pub reached_milestones: HashSet<(u8, AffinityMilestone)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AffinityMilestone {
    CoffeeInvite,   // affinity >= 15
    LunchTogether,  // affinity >= 30
    ProjectCollab,  // affinity >= 50, trust >= 30
    WorkBuddy,      // affinity >= 70, trust >= 50
}

impl Default for SocialGraphState {
    fn default() -> Self {
        Self {
            profiles: vec![
                CoworkerProfile {
                    id: 1,
                    codename: "Marta".to_string(),
                    role: CoworkerRole::Manager,
                    affinity: 2,
                    trust: 4,
                    personality: CoworkerPersonality::Ambitious,
                    backstory: "Rose through the ranks in record time. Never misses a deadline.",
                },
                CoworkerProfile {
                    id: 2,
                    codename: "Leo".to_string(),
                    role: CoworkerRole::Clerk,
                    affinity: 0,
                    trust: 0,
                    personality: CoworkerPersonality::Enthusiastic,
                    backstory: "Former barista who brings that same energy to inbox zero.",
                },
                CoworkerProfile {
                    id: 3,
                    codename: "Sana".to_string(),
                    role: CoworkerRole::Analyst,
                    affinity: 1,
                    trust: 1,
                    personality: CoworkerPersonality::Nurturing,
                    backstory: "The unofficial mentor who always has time to explain things.",
                },
                CoworkerProfile {
                    id: 4,
                    codename: "Ira".to_string(),
                    role: CoworkerRole::Coordinator,
                    affinity: -1,
                    trust: 0,
                    personality: CoworkerPersonality::Skeptical,
                    backstory: "Has seen three reorganizations. Trusts spreadsheets, not promises.",
                },
                CoworkerProfile {
                    id: 5,
                    codename: "Noah".to_string(),
                    role: CoworkerRole::Intern,
                    affinity: 0,
                    trust: -1,
                    personality: CoworkerPersonality::Reserved,
                    backstory: "Quiet observer taking notes. Surprisingly insightful when asked.",
                },
                CoworkerProfile {
                    id: 6,
                    codename: "Derek".to_string(),
                    role: CoworkerRole::Specialist,
                    affinity: 0,
                    trust: 0,
                    personality: CoworkerPersonality::Reserved,
                    backstory: "Expert in legacy systems. Prefers documentation to meetings.",
                },
                CoworkerProfile {
                    id: 7,
                    codename: "Jun".to_string(),
                    role: CoworkerRole::Analyst,
                    affinity: 0,
                    trust: 0,
                    personality: CoworkerPersonality::Ambitious,
                    backstory: "Transferred from a competitor. Hungry to prove their worth.",
                },
                CoworkerProfile {
                    id: 8,
                    codename: "Priya".to_string(),
                    role: CoworkerRole::Coordinator,
                    affinity: 0,
                    trust: 0,
                    personality: CoworkerPersonality::Nurturing,
                    backstory: "Organizes the team events. Remembers everyone's birthday.",
                },
            ],
            scenario_cursor: 0,
            reached_milestones: HashSet::new(),
        }
    }
}

impl SocialGraphState {
    pub fn normalize(&mut self) {
        self.profiles.sort_by_key(|profile| profile.id);
        self.profiles.dedup_by_key(|profile| profile.id);
        for profile in &mut self.profiles {
            profile.affinity = profile.affinity.clamp(-100, 100);
            profile.trust = profile.trust.clamp(-100, 100);
        }
    }

    pub fn manager_mut(&mut self) -> Option<&mut CoworkerProfile> {
        self.profiles
            .iter_mut()
            .find(|profile| profile.role == CoworkerRole::Manager)
    }

    pub fn manager(&self) -> Option<&CoworkerProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.role == CoworkerRole::Manager)
    }

    pub fn teammate_mut_by_id(&mut self, id: u8) -> Option<&mut CoworkerProfile> {
        self.profiles.iter_mut().find(|profile| profile.id == id)
    }

    pub fn average_affinity(&self) -> i32 {
        if self.profiles.is_empty() {
            return 0;
        }
        let total: i32 = self.profiles.iter().map(|p| p.affinity).sum();
        total / self.profiles.len() as i32
    }

    pub fn teammate_for_help(&self, seed: u64, day_number: u32, help_count: u32) -> Option<u8> {
        let teammate_ids = self
            .profiles
            .iter()
            .filter(|profile| profile.role != CoworkerRole::Manager)
            .map(|profile| profile.id)
            .collect::<Vec<_>>();
        if teammate_ids.is_empty() {
            return None;
        }

        let index = (seed
            .wrapping_add(day_number as u64 * 37)
            .wrapping_add(help_count as u64 * 17)
            % teammate_ids.len() as u64) as usize;
        teammate_ids.get(index).copied()
    }

    /// Compute the current manager arc stage based on the manager's trust and affinity.
    pub fn manager_arc_stage(&self) -> ManagerArcStage {
        let Some(manager) = self.manager() else {
            return ManagerArcStage::Stranger;
        };
        let trust = manager.trust;
        let affinity = manager.affinity;

        if trust >= 60 {
            if affinity >= 40 {
                ManagerArcStage::Ally
            } else if affinity < 0 {
                ManagerArcStage::Antagonist
            } else {
                // trust >= 60, 0 <= affinity < 40: treat as Mentor (closest fit)
                ManagerArcStage::Mentor
            }
        } else if trust >= 30 {
            if affinity >= 20 {
                ManagerArcStage::Mentor
            } else {
                ManagerArcStage::Evaluator
            }
        } else if trust >= 10 {
            ManagerArcStage::Acquaintance
        } else {
            ManagerArcStage::Stranger
        }
    }

    /// Check all coworkers for new affinity milestones.
    /// Returns a list of newly reached milestones (not previously in reached_milestones).
    #[allow(dead_code)]
    pub fn check_new_milestones(&mut self) -> Vec<(u8, AffinityMilestone)> {
        let mut new_milestones = Vec::new();
        for profile in &self.profiles {
            let id = profile.id;
            let affinity = profile.affinity;
            let trust = profile.trust;

            // Check each milestone in order
            let candidates = [
                (AffinityMilestone::CoffeeInvite, affinity >= 15, true),
                (AffinityMilestone::LunchTogether, affinity >= 30, true),
                (AffinityMilestone::ProjectCollab, affinity >= 50 && trust >= 30, true),
                (AffinityMilestone::WorkBuddy, affinity >= 70 && trust >= 50, true),
            ];

            for (milestone, condition, _) in candidates {
                if condition && !self.reached_milestones.contains(&(id, milestone)) {
                    new_milestones.push((id, milestone));
                }
            }
        }

        // Add to reached_milestones
        for &(id, milestone) in &new_milestones {
            self.reached_milestones.insert((id, milestone));
        }

        new_milestones
    }
}

#[derive(Resource, Debug, Clone)]
pub struct OfficeEconomyRules {
    pub base_salary_per_task: i32,
    pub failure_penalty_per_task: i32,
    pub level_salary_bonus: i32,
    pub streak_bonus_per_day: i32,
    pub max_streak_bonus_days: u32,
    pub burnout_stress_threshold: i32,
    pub burnout_salary_penalty: i32,
    pub xp_per_completed_task: u32,
    pub xp_penalty_per_failed_task: u32,
    pub xp_per_manager_checkin: u32,
    pub xp_per_coworker_help: u32,
    pub reputation_per_diplomacy_perk: i32,
    pub stress_relief_per_resilience_perk: i32,
    pub process_energy_discount_per_efficiency_perk: i32,
    pub max_perk_level: u8,
}

impl Default for OfficeEconomyRules {
    fn default() -> Self {
        Self {
            base_salary_per_task: 12,
            failure_penalty_per_task: 6,
            level_salary_bonus: 5,
            streak_bonus_per_day: 3,
            max_streak_bonus_days: 5,
            burnout_stress_threshold: 78,
            burnout_salary_penalty: 10,
            xp_per_completed_task: 8,
            xp_penalty_per_failed_task: 5,
            xp_per_manager_checkin: 3,
            xp_per_coworker_help: 2,
            reputation_per_diplomacy_perk: 1,
            stress_relief_per_resilience_perk: 2,
            process_energy_discount_per_efficiency_perk: 2,
            max_perk_level: 5,
        }
    }
}

impl OfficeEconomyRules {
    pub fn normalize(&mut self) {
        self.base_salary_per_task = self.base_salary_per_task.max(0);
        self.failure_penalty_per_task = self.failure_penalty_per_task.max(0);
        self.level_salary_bonus = self.level_salary_bonus.max(0);
        self.streak_bonus_per_day = self.streak_bonus_per_day.max(0);
        self.max_streak_bonus_days = self.max_streak_bonus_days.max(1);
        self.burnout_stress_threshold = self.burnout_stress_threshold.clamp(0, 100);
        self.burnout_salary_penalty = self.burnout_salary_penalty.max(0);
        self.xp_per_completed_task = self.xp_per_completed_task.max(1);
        self.xp_per_manager_checkin = self.xp_per_manager_checkin.max(1);
        self.xp_per_coworker_help = self.xp_per_coworker_help.max(1);
        self.reputation_per_diplomacy_perk = self.reputation_per_diplomacy_perk.max(0);
        self.stress_relief_per_resilience_perk = self.stress_relief_per_resilience_perk.max(0);
        self.process_energy_discount_per_efficiency_perk =
            self.process_energy_discount_per_efficiency_perk.max(0);
        self.max_perk_level = self.max_perk_level.max(1);
    }
}

#[derive(Resource, Debug, Clone)]
pub struct CareerProgression {
    pub level: u32,
    pub xp: u32,
    pub success_streak: u32,
    pub burnout_days: u32,
    pub efficiency_perk: u8,
    pub resilience_perk: u8,
    pub diplomacy_perk: u8,
}

impl Default for CareerProgression {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0,
            success_streak: 0,
            burnout_days: 0,
            efficiency_perk: 0,
            resilience_perk: 0,
            diplomacy_perk: 0,
        }
    }
}

impl CareerProgression {
    pub fn xp_for_next_level(&self) -> u32 {
        32 + self.level.saturating_sub(1) * 16
    }

    pub fn process_energy_discount(&self, economy: &OfficeEconomyRules) -> i32 {
        self.efficiency_perk as i32 * economy.process_energy_discount_per_efficiency_perk
    }

    pub fn reputation_bonus(&self, economy: &OfficeEconomyRules) -> i32 {
        self.diplomacy_perk as i32 * economy.reputation_per_diplomacy_perk
    }

    pub fn stress_relief_bonus(&self, economy: &OfficeEconomyRules) -> i32 {
        self.resilience_perk as i32 * economy.stress_relief_per_resilience_perk
    }

    pub fn add_experience(&mut self, gained_xp: u32, economy: &OfficeEconomyRules) -> u32 {
        self.xp = self.xp.saturating_add(gained_xp);
        let mut levels_gained = 0;
        while self.xp >= self.xp_for_next_level() {
            let threshold = self.xp_for_next_level();
            self.xp -= threshold;
            self.level = self.level.saturating_add(1);
            levels_gained += 1;
            self.apply_auto_perk_for_level(economy);
        }
        levels_gained
    }

    fn apply_auto_perk_for_level(&mut self, economy: &OfficeEconomyRules) {
        let cycle = self.level.saturating_sub(2) % 3;
        match cycle {
            0 => self.efficiency_perk = self.efficiency_perk.saturating_add(1),
            1 => self.resilience_perk = self.resilience_perk.saturating_add(1),
            _ => self.diplomacy_perk = self.diplomacy_perk.saturating_add(1),
        }
        self.efficiency_perk = self.efficiency_perk.min(economy.max_perk_level);
        self.resilience_perk = self.resilience_perk.min(economy.max_perk_level);
        self.diplomacy_perk = self.diplomacy_perk.min(economy.max_perk_level);
    }

    pub fn normalize(&mut self, economy: &OfficeEconomyRules) {
        self.level = self.level.max(1);
        self.efficiency_perk = self.efficiency_perk.min(economy.max_perk_level);
        self.resilience_perk = self.resilience_perk.min(economy.max_perk_level);
        self.diplomacy_perk = self.diplomacy_perk.min(economy.max_perk_level);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct UnlockCatalogState {
    pub quick_coffee: bool,
    pub efficient_processing: bool,
    pub conflict_training: bool,
    pub escalation_license: bool,
}

impl UnlockCatalogState {
    pub fn sync_with_progression(&mut self, progression: &CareerProgression) {
        self.quick_coffee = progression.level >= 2;
        self.efficient_processing = progression.level >= 3;
        self.conflict_training = progression.level >= 4;
        self.escalation_license = progression.level >= 5;
    }

    pub fn process_progress_multiplier(&self) -> f32 {
        if self.efficient_processing {
            1.18
        } else {
            1.0
        }
    }

    pub fn coffee_minutes(&self, base_minutes: u32) -> u32 {
        if self.quick_coffee {
            (base_minutes / 2).max(5)
        } else {
            base_minutes
        }
    }

    pub fn calm_focus_bonus(&self) -> i32 {
        if self.conflict_training {
            3
        } else {
            0
        }
    }

    pub fn calm_stress_relief_bonus(&self) -> i32 {
        if self.conflict_training {
            2
        } else {
            0
        }
    }

    pub fn reputation_bonus(&self) -> i32 {
        if self.escalation_license {
            1
        } else {
            0
        }
    }

    pub fn unlocked_count(&self) -> u32 {
        self.quick_coffee as u32
            + self.efficient_processing as u32
            + self.conflict_training as u32
            + self.escalation_license as u32
    }
}

#[allow(dead_code)]
#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveInterruptionContext {
    pub kind: Option<InterruptionKind>,
    pub coworker_name: Option<String>,
    pub description: String,
}

#[derive(Resource, Debug, Default)]
pub struct DayStats {
    pub processed_items: u32,
    pub coffee_breaks: u32,
    pub wait_actions: u32,
    pub failed_process_attempts: u32,
    pub interruptions_triggered: u32,
    pub calm_responses: u32,
    pub panic_responses: u32,
    pub manager_checkins: u32,
    pub coworker_helps: u32,
}

#[derive(Resource, Default)]
pub struct WorkerSpriteData {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

#[derive(Resource, Default)]
pub struct OfficeFontHandle(pub Handle<Font>);

pub fn load_office_font(mut font: ResMut<OfficeFontHandle>, asset_server: Res<AssetServer>) {
    font.0 = asset_server.load("fonts/sprout_lands.ttf");
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MilestoneKind {
    Friendly,    // affinity >= 25
    Trusted,     // trust >= 25
    CloseFriend, // affinity >= 50
    DeepTrust,   // trust >= 50
    Rival,       // affinity <= -25
    Distrusted,  // trust <= -25
}

#[derive(Resource, Debug, Clone, Default)]
pub struct FiredMilestones {
    pub fired: HashSet<(u8, MilestoneKind)>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ToastState {
    pub message: String,
    pub remaining_ticks: u32,
}

pub fn format_clock(total_minutes: u32) -> String {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours:02}:{minutes:02}")
}
