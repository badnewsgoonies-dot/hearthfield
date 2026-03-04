use std::collections::HashSet;

use bevy::prelude::*;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoworkerProfile {
    pub id: u8,
    pub codename: String,
    pub role: CoworkerRole,
    pub affinity: i32,
    pub trust: i32,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SocialGraphState {
    pub profiles: Vec<CoworkerProfile>,
    pub scenario_cursor: u32,
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
                },
                CoworkerProfile {
                    id: 2,
                    codename: "Leo".to_string(),
                    role: CoworkerRole::Clerk,
                    affinity: 0,
                    trust: 0,
                },
                CoworkerProfile {
                    id: 3,
                    codename: "Sana".to_string(),
                    role: CoworkerRole::Analyst,
                    affinity: 1,
                    trust: 1,
                },
                CoworkerProfile {
                    id: 4,
                    codename: "Ira".to_string(),
                    role: CoworkerRole::Coordinator,
                    affinity: -1,
                    trust: 0,
                },
                CoworkerProfile {
                    id: 5,
                    codename: "Noah".to_string(),
                    role: CoworkerRole::Intern,
                    affinity: 0,
                    trust: -1,
                },
            ],
            scenario_cursor: 0,
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

pub fn format_clock(total_minutes: u32) -> String {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours:02}:{minutes:02}")
}
