// || shree ganesh ||
//

use crate::interaction::apply_interaction_result as apply_interaction; //for testing purpose we are importing this apply_interaction_result
use crate::net::{Net, NodeId};

// it  is more a kind of execution engine while the net and interactions are the files stating the structure and the rules of the engine
// the runtime here is reponsible for implementing the core evaluation strategies i.e. to finding and processing redexes until no more redexes exist

// below is the struct of runtime
pub struct Runtime {
    net: Net, //this is the net that is taken under evaluation
    stats: RuntimeStats,
}

/// Defining the RuntimeStats struct
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    pub steps: usize, //this states how many interactions have been applied
    pub max_nodes: usize,
}

impl Runtime {
    //this function takes the ownership of the net and returns the Runtime with stats.
    pub fn new(net: Net) -> Self {
        Self {
            net,
            stats: RuntimeStats::default(),
        }
    }

    //Theory: There's mut self inside the args as it will be mutating both the runtime.net and runtime.stats
    pub fn run(&mut self) -> Result<(), String> {
        self.stats = RuntimeStats::default();
        self.stats.max_nodes = self.net.node_count();

        // Keep evaluating until no more redexes
        while let Some(redex) = self.net.next_redex() {
            // Apply the interaction rule
            match apply_interaction(&mut self.net, redex) {
                Ok(_) => {
                    self.stats.steps += 1;
                    self.stats.max_nodes = self.stats.max_nodes.max(self.net.node_count());
                }
                Err(msg) => {
                    return Err(format!("Error during evaluation: {}", msg));
                }
            }
        }

        Ok(())
    }

    // Run a single step of evaluation
    pub fn step(&mut self) -> Result<bool, String> {
        if let Some(redex) = self.net.next_redex() {
            match apply_interaction(&mut self.net, redex) {
                Ok(_) => {
                    self.stats.steps += 1;
                    self.stats.max_nodes = self.stats.max_nodes.max(self.net.node_count());
                    Ok(true) // More steps might be possible
                }
                Err(msg) => Err(format!("Error during step: {}", msg)),
            }
        } else {
            Ok(false) // No more steps possible
        }
    }

    // Get the current net
    pub fn get_net(&self) -> &Net {
        &self.net
    }

    // Get a mutable reference to the net
    pub fn get_net_mut(&mut self) -> &mut Net {
        &mut self.net
    }

    // Get computation statistics
    pub fn get_stats(&self) -> &RuntimeStats {
        &self.stats
    }
}
