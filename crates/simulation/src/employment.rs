use serde::{Deserialize, Serialize};

use crate::company::JobOffer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employment {
    pub agent_id: String,
    pub company_id: String,
    pub role: String,
    pub wage: f64,
    pub start_tick: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaborMarketState {
    pub employments: Vec<Employment>,
    pub job_openings: Vec<JobOffer>,
    pub total_wages_paid: f64,
    pub total_hires: u64,
    pub total_fires: u64,
}

impl LaborMarketState {
    pub fn new() -> Self {
        Self {
            employments: Vec::new(),
            job_openings: Vec::new(),
            total_wages_paid: 0.0,
            total_hires: 0,
            total_fires: 0,
        }
    }

    pub fn active_employments(&self) -> Vec<&Employment> {
        self.employments.iter().filter(|e| e.active).collect()
    }

    pub fn employments_for_agent(&self, agent_id: &str) -> Vec<&Employment> {
        self.employments
            .iter()
            .filter(|e| e.agent_id == agent_id && e.active)
            .collect()
    }

    pub fn employments_for_company(&self, company_id: &str) -> Vec<&Employment> {
        self.employments
            .iter()
            .filter(|e| e.company_id == company_id && e.active)
            .collect()
    }

    pub fn is_agent_employed(&self, agent_id: &str) -> bool {
        self.employments
            .iter()
            .any(|e| e.agent_id == agent_id && e.active)
    }

    pub fn is_agent_employed_at(&self, agent_id: &str, company_id: &str) -> bool {
        self.employments
            .iter()
            .any(|e| e.agent_id == agent_id && e.company_id == company_id && e.active)
    }

    pub fn employee_count_for_company(&self, company_id: &str) -> usize {
        self.employments
            .iter()
            .filter(|e| e.company_id == company_id && e.active)
            .count()
    }

    pub fn total_wage_cost_for_company(&self, company_id: &str) -> f64 {
        self.employments
            .iter()
            .filter(|e| e.company_id == company_id && e.active)
            .map(|e| e.wage)
            .sum()
    }

    pub fn working_employees_for_company(&self, company_id: &str) -> Vec<&Employment> {
        self.employments
            .iter()
            .filter(|e| e.company_id == company_id && e.active)
            .collect()
    }

    pub fn average_wage(&self) -> f64 {
        let active = self.active_employments();
        if active.is_empty() {
            return 0.0;
        }
        let total: f64 = active.iter().map(|e| e.wage).sum();
        total / active.len() as f64
    }

    pub fn median_wage(&self) -> f64 {
        let mut wages: Vec<f64> = self.active_employments().iter().map(|e| e.wage).collect();
        if wages.is_empty() {
            return 0.0;
        }
        wages.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = wages.len() / 2;
        if wages.len() % 2 == 0 {
            (wages[mid - 1] + wages[mid]) / 2.0
        } else {
            wages[mid]
        }
    }

    pub fn unemployed_count(&self, total_agents: usize) -> usize {
        let employed = self.active_employments().len();
        total_agents.saturating_sub(employed)
    }

    pub fn open_positions(&self) -> u32 {
        self.job_openings.iter().map(|o| o.openings).sum()
    }

    pub fn create_employment(
        &mut self,
        agent_id: String,
        company_id: String,
        role: String,
        wage: f64,
        tick: u64,
    ) -> bool {
        if self.is_agent_employed(&agent_id) {
            return false;
        }
        self.employments.push(Employment {
            agent_id,
            company_id,
            role,
            wage,
            start_tick: tick,
            active: true,
        });
        self.total_hires += 1;
        true
    }

    pub fn terminate_employment(&mut self, agent_id: &str, company_id: &str) -> bool {
        if let Some(emp) = self
            .employments
            .iter_mut()
            .find(|e| e.agent_id == agent_id && e.company_id == company_id && e.active)
        {
            emp.active = false;
            self.total_fires += 1;
            true
        } else {
            false
        }
    }

    pub fn terminate_all_for_company(&mut self, company_id: &str) -> Vec<String> {
        let mut fired = Vec::new();
        for emp in &mut self.employments {
            if emp.company_id == company_id && emp.active {
                emp.active = false;
                fired.push(emp.agent_id.clone());
                self.total_fires += 1;
            }
        }
        fired
    }

    pub fn compute_job_score(wage: f64, skill_match: f64, location_match: bool) -> f64 {
        let wage_score = wage / 50.0;
        let location_bonus = if location_match { 0.1 } else { 0.0 };
        wage_score + skill_match + location_bonus
    }

    pub fn highest_paying_company(&self) -> Option<(String, f64)> {
        let mut best: Option<(String, f64)> = None;
        let mut seen = std::collections::HashSet::new();
        for emp in &self.employments {
            if emp.active && seen.insert(emp.company_id.clone()) {
                match &best {
                    Some((_, max_wage)) if emp.wage <= *max_wage => {}
                    _ => {
                        best = Some((emp.company_id.clone(), emp.wage));
                    }
                }
            }
        }
        best
    }

    pub fn largest_employer(&self) -> Option<(String, usize)> {
        let mut counts = std::collections::HashMap::new();
        for emp in &self.employments {
            if emp.active {
                *counts.entry(emp.company_id.clone()).or_insert(0) += 1;
            }
        }
        counts.into_iter().max_by_key(|(_, c)| *c)
    }
}
