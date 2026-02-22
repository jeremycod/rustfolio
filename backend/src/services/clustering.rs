/// Clustering module for identifying correlated asset groups
///
/// This module implements a hierarchical clustering algorithm to group assets
/// based on their correlation similarity. It uses a simple threshold-based
/// approach that is computationally efficient and produces interpretable results.

use crate::models::risk::{AssetCluster, CorrelationMatrix};
use std::collections::HashMap;

/// Identify clusters of correlated assets using correlation-based clustering.
///
/// This function uses a threshold-based clustering approach to group
/// assets based on their correlation similarity. The algorithm automatically
/// determines the number of clusters based on portfolio size.
///
/// # Arguments
/// * `matrix` - The correlation matrix containing tickers and correlations
///
/// # Returns
/// Tuple of (clusters, cluster_labels, inter_cluster_correlations)
pub fn identify_correlation_clusters(
    matrix: &CorrelationMatrix,
) -> (
    Vec<AssetCluster>,
    HashMap<String, usize>,
    Vec<Vec<f64>>,
) {
    let n = matrix.tickers.len();

    // Need at least 2 tickers for clustering
    if n < 2 {
        return (Vec::new(), HashMap::new(), Vec::new());
    }

    // Determine optimal number of clusters based on portfolio size
    let target_clusters = if n <= 4 {
        2 // Small portfolios: 2 clusters
    } else if n <= 8 {
        3 // Medium portfolios: 3 clusters
    } else if n <= 12 {
        4 // Larger portfolios: 4 clusters
    } else {
        5 // Very large portfolios: 5 clusters
    };

    // Perform simple correlation-based clustering
    let cluster_assignments = simple_correlation_clustering(&matrix.matrix_2d, target_clusters);

    // Build cluster groups
    let mut cluster_map: HashMap<usize, Vec<String>> = HashMap::new();
    for (idx, &cluster_id) in cluster_assignments.iter().enumerate() {
        cluster_map
            .entry(cluster_id)
            .or_insert_with(Vec::new)
            .push(matrix.tickers[idx].clone());
    }

    // Calculate cluster statistics
    let clusters = cluster_map
        .iter()
        .map(|(&cluster_id, tickers)| {
            let avg_correlation = calculate_cluster_avg_correlation(&matrix.matrix_2d, tickers, &matrix.tickers);
            let color = get_cluster_color(cluster_id);
            let name = generate_cluster_name(cluster_id, tickers.len());

            AssetCluster {
                cluster_id,
                tickers: tickers.clone(),
                avg_correlation,
                color,
                name,
            }
        })
        .collect::<Vec<_>>();

    // Build cluster labels map
    let mut cluster_labels = HashMap::new();
    for cluster in &clusters {
        for ticker in &cluster.tickers {
            cluster_labels.insert(ticker.clone(), cluster.cluster_id);
        }
    }

    // Calculate inter-cluster correlations
    let inter_cluster_corr = calculate_inter_cluster_correlations(&clusters, &matrix.matrix_2d, &matrix.tickers);

    (clusters, cluster_labels, inter_cluster_corr)
}

/// Simple correlation-based clustering using threshold method.
///
/// Groups assets by finding the most correlated pairs first, then expanding clusters.
/// This approach is efficient and produces interpretable results.
fn simple_correlation_clustering(matrix: &[Vec<f64>], target_clusters: usize) -> Vec<usize> {
    let n = matrix.len();
    let mut assignments = vec![usize::MAX; n]; // Initialize with "unassigned"
    let mut next_cluster_id = 0;

    // Start with the first unassigned ticker as a seed
    for seed_idx in 0..n {
        if assignments[seed_idx] != usize::MAX {
            continue; // Already assigned
        }

        // Assign seed to a new cluster
        assignments[seed_idx] = next_cluster_id;

        // Find all tickers highly correlated with this seed (correlation > 0.6)
        for other_idx in 0..n {
            if seed_idx == other_idx || assignments[other_idx] != usize::MAX {
                continue;
            }

            let correlation = matrix[seed_idx][other_idx];
            if correlation > 0.6 {
                assignments[other_idx] = next_cluster_id;
            }
        }

        next_cluster_id += 1;

        // Stop if we have enough clusters
        if next_cluster_id >= target_clusters {
            break;
        }
    }

    // Assign any remaining unassigned tickers to the nearest cluster
    for idx in 0..n {
        if assignments[idx] == usize::MAX {
            // Find cluster with highest average correlation
            let mut best_cluster = 0;
            let mut best_avg_corr = f64::NEG_INFINITY;

            for cluster_id in 0..next_cluster_id {
                let cluster_indices: Vec<usize> = assignments
                    .iter()
                    .enumerate()
                    .filter(|(_, &c)| c == cluster_id)
                    .map(|(i, _)| i)
                    .collect();

                if cluster_indices.is_empty() {
                    continue;
                }

                let avg_corr: f64 = cluster_indices
                    .iter()
                    .map(|&i| matrix[idx][i])
                    .sum::<f64>() / cluster_indices.len() as f64;

                if avg_corr > best_avg_corr {
                    best_avg_corr = avg_corr;
                    best_cluster = cluster_id;
                }
            }

            assignments[idx] = best_cluster;
        }
    }

    assignments
}

/// Calculate average intra-cluster correlation
fn calculate_cluster_avg_correlation(
    matrix: &[Vec<f64>],
    cluster_tickers: &[String],
    all_tickers: &[String],
) -> f64 {
    if cluster_tickers.len() < 2 {
        return 1.0; // Single ticker cluster has perfect self-correlation
    }

    // Get indices of tickers in this cluster
    let indices: Vec<usize> = cluster_tickers
        .iter()
        .filter_map(|ticker| all_tickers.iter().position(|t| t == ticker))
        .collect();

    // Calculate average correlation between all pairs in the cluster
    let mut sum = 0.0;
    let mut count = 0;

    for i in 0..indices.len() {
        for j in (i + 1)..indices.len() {
            sum += matrix[indices[i]][indices[j]];
            count += 1;
        }
    }

    if count == 0 {
        1.0
    } else {
        sum / count as f64
    }
}

/// Calculate correlation matrix between cluster centroids
fn calculate_inter_cluster_correlations(
    clusters: &[AssetCluster],
    matrix: &[Vec<f64>],
    all_tickers: &[String],
) -> Vec<Vec<f64>> {
    let num_clusters = clusters.len();
    let mut inter_cluster = vec![vec![0.0; num_clusters]; num_clusters];

    for i in 0..num_clusters {
        for j in 0..num_clusters {
            if i == j {
                inter_cluster[i][j] = 1.0; // Cluster with itself
            } else {
                // Calculate average correlation between tickers in cluster i and cluster j
                let cluster_i_indices: Vec<usize> = clusters[i]
                    .tickers
                    .iter()
                    .filter_map(|ticker| all_tickers.iter().position(|t| t == ticker))
                    .collect();

                let cluster_j_indices: Vec<usize> = clusters[j]
                    .tickers
                    .iter()
                    .filter_map(|ticker| all_tickers.iter().position(|t| t == ticker))
                    .collect();

                let mut sum = 0.0;
                let mut count = 0;

                for &idx_i in &cluster_i_indices {
                    for &idx_j in &cluster_j_indices {
                        sum += matrix[idx_i][idx_j];
                        count += 1;
                    }
                }

                inter_cluster[i][j] = if count > 0 { sum / count as f64 } else { 0.0 };
            }
        }
    }

    inter_cluster
}

/// Get a distinct color for each cluster (hex format)
fn get_cluster_color(cluster_id: usize) -> String {
    let colors = [
        "#3B82F6", // Blue
        "#10B981", // Green
        "#F59E0B", // Amber
        "#EF4444", // Red
        "#8B5CF6", // Purple
        "#EC4899", // Pink
        "#14B8A6", // Teal
        "#F97316", // Orange
    ];

    colors[cluster_id % colors.len()].to_string()
}

/// Generate a descriptive name for a cluster
fn generate_cluster_name(cluster_id: usize, size: usize) -> String {
    let cluster_letter = ('A' as u8 + cluster_id as u8) as char;
    format!("Cluster {} ({} assets)", cluster_letter, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::risk::{CorrelationMatrix, CorrelationPair};

    #[test]
    fn test_clustering_with_high_correlation() {
        // Create a simple correlation matrix with 4 tickers
        // AAPL and MSFT are highly correlated (0.9)
        // GOOGL and META are highly correlated (0.85)
        let matrix = CorrelationMatrix {
            portfolio_id: "test".to_string(),
            tickers: vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(), "META".to_string()],
            correlations: vec![
                CorrelationPair { ticker1: "AAPL".to_string(), ticker2: "MSFT".to_string(), correlation: 0.9 },
                CorrelationPair { ticker1: "AAPL".to_string(), ticker2: "GOOGL".to_string(), correlation: 0.3 },
                CorrelationPair { ticker1: "AAPL".to_string(), ticker2: "META".to_string(), correlation: 0.2 },
                CorrelationPair { ticker1: "MSFT".to_string(), ticker2: "GOOGL".to_string(), correlation: 0.25 },
                CorrelationPair { ticker1: "MSFT".to_string(), ticker2: "META".to_string(), correlation: 0.3 },
                CorrelationPair { ticker1: "GOOGL".to_string(), ticker2: "META".to_string(), correlation: 0.85 },
            ],
            matrix_2d: vec![
                vec![1.0, 0.9, 0.3, 0.2],
                vec![0.9, 1.0, 0.25, 0.3],
                vec![0.3, 0.25, 1.0, 0.85],
                vec![0.2, 0.3, 0.85, 1.0],
            ],
            clusters: None,
            cluster_labels: None,
            inter_cluster_correlations: None,
        };

        let (clusters, cluster_labels, _inter_cluster_corr) = identify_correlation_clusters(&matrix);

        // Should identify 2 clusters
        assert_eq!(clusters.len(), 2);

        // Check that cluster labels exist for all tickers
        assert_eq!(cluster_labels.len(), 4);

        // Check that AAPL and MSFT are in the same cluster (high correlation)
        let aapl_cluster = cluster_labels.get("AAPL").unwrap();
        let msft_cluster = cluster_labels.get("MSFT").unwrap();
        assert_eq!(aapl_cluster, msft_cluster);

        // Check that GOOGL and META are in the same cluster
        let googl_cluster = cluster_labels.get("GOOGL").unwrap();
        let meta_cluster = cluster_labels.get("META").unwrap();
        assert_eq!(googl_cluster, meta_cluster);

        // Check that the two clusters are different
        assert_ne!(aapl_cluster, googl_cluster);
    }

    #[test]
    fn test_clustering_with_single_ticker() {
        let matrix = CorrelationMatrix {
            portfolio_id: "test".to_string(),
            tickers: vec!["AAPL".to_string()],
            correlations: vec![],
            matrix_2d: vec![vec![1.0]],
            clusters: None,
            cluster_labels: None,
            inter_cluster_correlations: None,
        };

        let (clusters, cluster_labels, inter_cluster_corr) = identify_correlation_clusters(&matrix);

        // Should return empty results for single ticker
        assert_eq!(clusters.len(), 0);
        assert_eq!(cluster_labels.len(), 0);
        assert_eq!(inter_cluster_corr.len(), 0);
    }

    #[test]
    fn test_cluster_color_assignment() {
        // Test that colors are consistently assigned
        assert_eq!(get_cluster_color(0), "#3B82F6");
        assert_eq!(get_cluster_color(1), "#10B981");
        assert_eq!(get_cluster_color(2), "#F59E0B");
        assert_eq!(get_cluster_color(8), "#3B82F6"); // Wraps around
    }

    #[test]
    fn test_cluster_name_generation() {
        assert_eq!(generate_cluster_name(0, 3), "Cluster A (3 assets)");
        assert_eq!(generate_cluster_name(1, 5), "Cluster B (5 assets)");
        assert_eq!(generate_cluster_name(2, 1), "Cluster C (1 assets)");
    }
}
