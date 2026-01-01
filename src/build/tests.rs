//! Comprehensive tests for build orchestration

use crate::build::dependency::DependencyGraph;
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph_build_order() {
        let graph = DependencyGraph::new("BTCDecoded".to_string());
        let order = graph.get_build_order().unwrap();

        // Verify blvm-consensus comes first (no dependencies)
        let consensus_pos = order.iter().position(|r| r == "blvm-consensus").unwrap();
        assert_eq!(consensus_pos, 0, "blvm-consensus should be first");

        // Verify blvm-protocol comes after blvm-consensus
        let protocol_pos = order.iter().position(|r| r == "blvm-protocol").unwrap();
        assert!(
            protocol_pos > consensus_pos,
            "blvm-protocol should come after blvm-consensus"
        );

        // Verify blvm-node comes after blvm-protocol
        let node_pos = order.iter().position(|r| r == "blvm-node").unwrap();
        assert!(
            node_pos > protocol_pos,
            "blvm-node should come after blvm-protocol"
        );
    }

    #[test]
    fn test_dependency_graph_no_circular_dependencies() {
        let graph = DependencyGraph::new("BTCDecoded".to_string());
        let order = graph.get_build_order();

        assert!(order.is_ok(), "Should not have circular dependencies");
    }

    #[test]
    fn test_dependency_graph_all_repos_included() {
        let graph = DependencyGraph::new("BTCDecoded".to_string());
        let order = graph.get_build_order().unwrap();
        let repos = graph.repositories();

        // All repos should be in build order
        for repo in &repos {
            assert!(
                order.contains(repo),
                "Repository {} should be in build order",
                repo
            );
        }

        // Build order should not have duplicates
        let unique: HashSet<_> = order.iter().collect();
        assert_eq!(
            unique.len(),
            order.len(),
            "Build order should not have duplicates"
        );
    }

    #[test]
    fn test_dependency_graph_dependencies_respected() {
        let graph = DependencyGraph::new("BTCDecoded".to_string());
        let order = graph.get_build_order().unwrap();

        // Check that dependencies come before dependents
        for (repo, deps) in [
            ("blvm-protocol", vec!["blvm-consensus"]),
            ("blvm-node", vec!["blvm-protocol", "blvm-consensus"]),
            ("blvm-commons", vec!["blvm-sdk"]),
            ("blvm", vec!["blvm-node"]),
        ] {
            let repo_pos = order.iter().position(|r| r == repo).unwrap();
            for dep in deps {
                let dep_pos = order.iter().position(|r| r == dep).unwrap();
                assert!(
                    dep_pos < repo_pos,
                    "Dependency {} should come before {}",
                    dep,
                    repo
                );
            }
        }
    }

    #[test]
    fn test_parallel_groups() {
        let graph = DependencyGraph::new("BTCDecoded".to_string());
        let groups = graph.get_parallel_groups().unwrap();

        // First group should contain repos with no dependencies
        assert!(
            !groups.is_empty(),
            "Should have at least one parallel group"
        );

        // blvm-consensus and blvm-sdk can be built in parallel (no dependencies)
        let first_group = &groups[0];
        assert!(
            first_group.contains(&"blvm-consensus".to_string())
                || first_group.contains(&"blvm-sdk".to_string()),
            "First group should contain repos with no dependencies"
        );
    }
}
