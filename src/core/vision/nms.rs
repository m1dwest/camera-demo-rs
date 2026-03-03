use crate::core::vision::Detection;

#[derive(Debug, Clone, Copy)]
pub enum SoftNmsMethod {
    Linear,
    Gaussian,
}

#[derive(Debug, Clone, Copy)]
pub struct SoftNmsParams {
    pub iou_thresh: f32,   // used by Linear; can be ~0.3..0.6
    pub sigma: f32,        // used by Gaussian; typical ~0.5
    pub score_thresh: f32, // drop boxes once score falls below this
    pub method: SoftNmsMethod,
    pub class_aware: bool,
    pub max_detections: usize,
}

/// Soft-NMS (greedy) that returns up to `max_detections` boxes.
/// Complexity: O(n^2), but with small n it's trivial.
pub fn soft_nms(mut dets: Vec<Detection>, p: SoftNmsParams) -> Vec<Detection> {
    // We’ll pick the current best, then decay the rest, repeat.
    let mut out: Vec<Detection> = Vec::with_capacity(p.max_detections);

    // Optional: remove NaN/negative scores early
    dets.retain(|d| d.score.is_finite() && d.score > 0.0);

    while !dets.is_empty() && out.len() < p.max_detections {
        // 1) Select best remaining by score
        let mut best_idx = 0usize;
        for i in 1..dets.len() {
            if dets[i].score > dets[best_idx].score {
                best_idx = i;
            }
        }
        let best = dets.swap_remove(best_idx);

        // If best is already below threshold, we can stop (nothing else will be better).
        if best.score < p.score_thresh {
            break;
        }

        // Keep it
        let best_rect = best.rect;
        let best_class = best.class_id;
        out.push(best);

        // 2) Decay scores of remaining boxes
        for d in &mut dets {
            if p.class_aware && d.class_id != best_class {
                continue;
            }

            let iou = best_rect.iou(&d.rect);
            if !iou.is_finite() {
                continue;
            }

            let weight = match p.method {
                SoftNmsMethod::Linear => {
                    if iou > p.iou_thresh {
                        (1.0 - iou).max(0.0)
                    } else {
                        1.0
                    }
                }
                SoftNmsMethod::Gaussian => {
                    // exp(-(iou^2)/sigma)
                    // sigma must be > 0; if not, fall back to hard-ish behavior
                    if p.sigma > 0.0 {
                        (-(iou * iou) / p.sigma).exp()
                    } else if iou > p.iou_thresh {
                        0.0
                    } else {
                        1.0
                    }
                }
            };

            d.score *= weight;
        }

        dets.retain(|d| d.score >= p.score_thresh);
    }

    out
}
