use super::{Digit, Flow, SegmentId, TransitionSpec};

impl SegmentId {
    /// Physical neighbors (segments that share an edge)
    const fn neighbors(self) -> &'static [SegmentId] {
        use SegmentId::*;
        match self {
            Top => &[TopRight, TopLeft, Middle],
            TopRight => &[Top, Middle, BottomRight],
            BottomRight => &[TopRight, Middle, Bottom],
            Bottom => &[BottomRight, BottomLeft, Middle],
            BottomLeft => &[Bottom, Middle, TopLeft],
            TopLeft => &[Top, Middle, BottomLeft],
            Middle => &[Top, TopRight, BottomRight, Bottom, BottomLeft, TopLeft],
        }
    }

    /// Distance heuristic between segments (0-3, where 0 = same, 1 = adjacent)
    fn distance_to(self, other: SegmentId) -> u8 {
        if self == other {
            return 0;
        }
        if self.neighbors().contains(&other) {
            return 1;
        }

        // BFS for longer paths
        use std::collections::{HashSet, VecDeque};
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((self, 0u8));
        visited.insert(self);

        while let Some((seg, dist)) = queue.pop_front() {
            if seg == other {
                return dist;
            }
            for &neighbor in seg.neighbors() {
                if visited.insert(neighbor) {
                    queue.push_back((neighbor, dist + 1));
                }
            }
        }
        3 // max distance
    }
}

impl TransitionSpec {
    pub fn compute_flows(from_digit: Digit, to_digit: Digit) -> Self {
        let from_segs: Vec<_> = from_digit.active_segments().collect();
        let to_segs: Vec<_> = to_digit.active_segments().collect();

        let mut flows = Vec::new();

        // 1. Handle disappearing segments (these MUST flow somewhere)
        for &from_seg in &from_segs {
            if !to_segs.contains(&from_seg) {
                // Segment disappears - find nearest active neighbor in target
                let targets_with_dist: Vec<_> = to_segs
                    .iter()
                    .map(|&to_seg| (to_seg, from_seg.distance_to(to_seg)))
                    .collect();

                let min_dist = targets_with_dist
                    .iter()
                    .map(|(_, dist)| *dist)
                    .min()
                    .expect("to_digit must have at least one active segment");

                let nearest: Vec<_> = targets_with_dist
                    .iter()
                    .filter(|(_, dist)| *dist == min_dist)
                    .map(|(seg, _)| *seg)
                    .collect();

                let share = 1.0 / nearest.len() as f32;
                for target in nearest {
                    flows.push(Flow {
                        from: from_seg,
                        to: target,
                        share,
                    });
                }
            }
        }

        // 2. NEW: Create "excitement flows" from stable segments to appearing segments
        // This makes boring transitions more dynamic
        let stable_segs: Vec<_> = from_segs
            .iter()
            .filter(|&&seg| to_segs.contains(&seg))
            .copied()
            .collect();

        let appearing_segs: Vec<_> = to_segs
            .iter()
            .filter(|&&seg| !from_segs.contains(&seg))
            .copied()
            .collect();

        // If there are segments appearing and stable segments exist,
        // create some "donation flows" for visual interest
        if !appearing_segs.is_empty() && !stable_segs.is_empty() {
            for &appearing_seg in &appearing_segs {
                // Find closest stable segment to donate some mass
                let mut closest_stable = stable_segs[0];
                let mut min_dist = closest_stable.distance_to(appearing_seg);

                for &stable_seg in &stable_segs {
                    let dist = stable_seg.distance_to(appearing_seg);
                    if dist < min_dist {
                        min_dist = dist;
                        closest_stable = stable_seg;
                    }
                }

                // Create a small donation flow (20% of mass)
                flows.push(Flow {
                    from: closest_stable,
                    to: appearing_seg,
                    share: 0.2, // Donate 20% - segment stays mostly intact
                });
            }
        }

        TransitionSpec {
            from_digit,
            to_digit,
            flows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbor_symmetry() {
        use SegmentId::*;
        let all_segments = [
            Top,
            TopRight,
            BottomRight,
            Bottom,
            BottomLeft,
            TopLeft,
            Middle,
        ];

        // If A is a neighbor of B, then B should be a neighbor of A
        for &seg_a in &all_segments {
            for &seg_b in seg_a.neighbors() {
                assert!(
                    seg_b.neighbors().contains(&seg_a),
                    "{:?} lists {:?} as neighbor, but {:?} doesn't list {:?}",
                    seg_a,
                    seg_b,
                    seg_b,
                    seg_a
                );
            }
        }
    }

    #[test]
    fn test_distance_to_self() {
        use SegmentId::*;
        let all_segments = [
            Top,
            TopRight,
            BottomRight,
            Bottom,
            BottomLeft,
            TopLeft,
            Middle,
        ];

        // Distance from any segment to itself should be 0
        for &seg in &all_segments {
            assert_eq!(
                seg.distance_to(seg),
                0,
                "Distance from {:?} to itself should be 0",
                seg
            );
        }
    }

    #[test]
    fn test_all_pairs_distances() {
        use SegmentId::*;

        // Exhaustive test of all pairs with expected distances
        let test_cases = vec![
            // Distance 0 (self) - already tested above

            // Distance 1 (direct neighbors)
            (Top, TopRight, 1),
            (Top, TopLeft, 1),
            (Top, Middle, 1),
            (TopRight, BottomRight, 1),
            (TopRight, Middle, 1),
            (BottomRight, Bottom, 1),
            (BottomRight, Middle, 1),
            (Bottom, BottomLeft, 1),
            (Bottom, Middle, 1),
            (BottomLeft, TopLeft, 1),
            (BottomLeft, Middle, 1),
            (TopLeft, Middle, 1),
            // Distance 2 (two hops)
            (Top, BottomRight, 2),
            (Top, Bottom, 2),
            (Top, BottomLeft, 2),
            (TopRight, TopLeft, 2),
            (TopRight, Bottom, 2),
            (TopRight, BottomLeft, 2),
            (BottomRight, BottomLeft, 2),
            (BottomRight, TopLeft, 2),
            (BottomLeft, TopRight, 2),
        ];

        for (seg_a, seg_b, expected_dist) in test_cases {
            assert_eq!(
                seg_a.distance_to(seg_b),
                expected_dist,
                "Distance from {:?} to {:?} should be {}",
                seg_a,
                seg_b,
                expected_dist
            );
            // Also test symmetry
            assert_eq!(
                seg_b.distance_to(seg_a),
                expected_dist,
                "Distance from {:?} to {:?} should be {} (symmetry)",
                seg_b,
                seg_a,
                expected_dist
            );
        }
    }

    #[test]
    fn test_max_flows_across_all_transitions() {
        let all_digits = [
            Digit::Zero,
            Digit::One,
            Digit::Two,
            Digit::Three,
            Digit::Four,
            Digit::Five,
            Digit::Six,
            Digit::Seven,
            Digit::Eight,
            Digit::Nine,
        ];

        let mut max_flows = 0;
        let mut max_transition = (Digit::Zero, Digit::Zero);

        println!("\n=== Flow counts for all digit transitions ===");
        for &from_digit in &all_digits {
            for &to_digit in &all_digits {
                let spec = TransitionSpec::compute_flows(from_digit, to_digit);
                let flow_count = spec.flows.len();

                if flow_count > max_flows {
                    max_flows = flow_count;
                    max_transition = (from_digit, to_digit);
                }

                println!("{:?} -> {:?}: {} flows", from_digit, to_digit, flow_count);
            }
        }

        println!("\n=== Summary ===");
        println!(
            "Maximum flows: {} (from {:?} to {:?})",
            max_flows, max_transition.0, max_transition.1
        );
        println!("Current MAX_FLOWS constant: 32");
        println!("Safety margin: {} unused slots", 32 - max_flows);

        // Assert that our MAX_FLOWS constant is sufficient
        assert!(
            max_flows <= 32,
            "MAX_FLOWS (32) is too small! Need at least {}",
            max_flows
        );
    }
}
