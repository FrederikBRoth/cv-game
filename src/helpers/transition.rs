use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::helpers::transition;
#[derive(Debug, Clone, PartialEq)]

pub enum ScrollTransitions {
    Home,
    CSharp,
    CPlusPLus,
    Rust,
    Containerization,
}

pub struct TransitionHandler<T> {
    pub transition_map: BTreeMap<i64, T>,
    last_transition: Option<T>,
}

impl<T: Clone + PartialEq> TransitionHandler<T> {
    pub fn new(transition_map: BTreeMap<i64, T>) -> Self {
        Self {
            transition_map,
            last_transition: None,
        }
    }

    pub fn trigger_transition(&mut self, position: i64) -> Option<T> {
        let mut start = 0;
        let mut transition: Option<T> = None;
        for (&n, value) in self.transition_map.iter() {
            if is_between(start, n, position) {
                transition = Some(value.clone());
                break;
            }
            start = n;
        }

        if let Some(last_trans) = &self.last_transition {
            if let Some(trans) = &transition {
                if last_trans == trans {
                    return None;
                }
            }
        }
        self.last_transition = transition.clone();
        transition
    }
}

fn is_between(start: i64, end: i64, number: i64) -> bool {
    return number >= start && end > number;
}
