use shipyard::Component;

#[derive(Clone, Debug, Default, Component)]
pub struct TimeRemap {
    pub curve: Vec<(i32, i32)>,
    pub freeze_frame: Option<i32>,
}

impl TimeRemap {
    pub fn resolve(&self, layer_frame: i32, fallback: i32) -> i32 {
        if let Some(frame) = self.freeze_frame {
            return frame;
        }
        if self.curve.is_empty() {
            return fallback;
        }
        match self.curve.binary_search_by_key(&layer_frame, |&(k, _)| k) {
            Ok(idx) => self.curve[idx].1,
            Err(0) => self.curve[0].1,
            Err(idx) if idx >= self.curve.len() => self.curve[self.curve.len() - 1].1,
            Err(idx) => {
                let (k0, v0) = self.curve[idx - 1];
                let (k1, v1) = self.curve[idx];
                if k1 == k0 {
                    v0
                } else {
                    let t = (layer_frame - k0) as f32 / (k1 - k0) as f32;
                    (v0 as f32 + (v1 - v0) as f32 * t).round() as i32
                }
            }
        }
    }
}
