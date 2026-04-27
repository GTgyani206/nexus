mod interaction;
mod net;
mod runtime;

use net::{Net, Node, Port};
use runtime::Runtime;

fn main() {
    // Create a simple program that adds two numbers: 2 + 3
    let mut net = Net::new();

    // First, create the constructor nodes for the numbers 2 and 3
    let two = net.create_node(Node::Con {
        tag: 2,
        ports: [Port::null(), Port::null()],
    });
    let three = net.create_node(Node::Con {
        tag: 3,
        ports: [Port::null(), Port::null()],
    });

    // Create an "Add" function node (represented as a constructor with tag 100)
    let add = net.create_node(Node::Con {
        tag: 100,
        ports: [Port::null(), Port::null()],
    });

    // Connect the numbers to the add function
    net.connect(add, 0, two, 0);
    net.connect(add, 1, three, 0);

    // Print the initial state
    println!("Initial net state:");
    println!("{:?}", net);

    // Create a runtime and run the computation
    let mut runtime = Runtime::new(net);

    // Option 1: Run to completion
    match runtime.run() {
        Ok(_) => {
            println!("\nComputation finished successfully!");
            println!("Steps taken: {}", runtime.get_stats().steps);
            println!("Maximum nodes: {}", runtime.get_stats().max_nodes);
            println!("\nFinal result:");
            println!("{:?}", runtime.get_net());
        }
        Err(e) => println!("Error during computation: {}", e),
    }

    // Option 2: Step through manually

    let mut step_count = 0;
    loop {
        match runtime.step() {
            Ok(more_steps) => {
                step_count += 1;
                println!("\nAfter step {}:", step_count);
                println!("{:?}", runtime.get_net());

                if !more_steps {
                    println!("\nNo more steps possible. Computation complete.");
                    break;
                }
            }
            Err(e) => {
                println!("Error during step {}: {}", step_count + 1, e);
                break;
            }
        }
    }
}
