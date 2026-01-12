//! 7-segment display transition logic with flow computation.

use super::digit::{Digit, Flow, SegmentId};

#[derive(Clone, Debug)]
pub struct TransitionSpec {
    pub from_digit: Digit,
    pub to_digit: Digit,
    pub flows: Vec<Flow>,
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

        // 2. Create "excitement flows" from stable segments to appearing segments
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
                    share: 0.2,
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

        for &from_digit in &all_digits {
            for &to_digit in &all_digits {
                let spec = TransitionSpec::compute_flows(from_digit, to_digit);
                max_flows = max_flows.max(spec.flows.len());
            }
        }

        // Assert that our MAX_FLOWS constant is sufficient
        assert!(
            max_flows <= 32,
            "MAX_FLOWS (32) is too small! Need at least {}",
            max_flows
        );
    }
}
