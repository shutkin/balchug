use balchug_common::settings::InertiaProperties;

#[derive(Copy, Clone)]
struct InertiaTarget {
    value: f32,
    is_permanent: bool,
}

#[derive(Copy, Clone)]
pub struct Inertia {
    limit_down: f32,
    limit_up: f32,
    value: f32,
    target: Option<InertiaTarget>,
    speed: f32,
    properties: InertiaProperties,
}

impl Inertia {
    pub fn new(value: f32, properties: InertiaProperties) -> Inertia {
        Inertia {
            value,
            target: None,
            speed: 0.0,
            limit_down: 0.0,
            limit_up: f32::INFINITY,
            properties,
        }
    }

    pub fn set_limit_up(&mut self, limit_up: f32) {
        self.limit_up = limit_up;
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_target(&mut self, target: f32, is_permanent: bool) {
        self.target = Some(InertiaTarget { value: target, is_permanent });
    }

    pub fn clear_target(&mut self) {
        self.target = None;
    }

    pub fn has_permanent_target(&self) -> bool {
        if let Some(target) = self.target {
            target.is_permanent
        } else {
            false
        }
    }

    pub fn set_properties(&mut self, properties: InertiaProperties) {
        self.properties = properties;
    }

    pub fn live(&mut self, elapsed: f32) -> (bool, f32) {
        let prev_value = self.value;
        let mut target_sign_before = false;
        if let Some(target) = self.target {
            target_sign_before = target.value > self.value;
            let target_factor = 600.0 / (1.0 + self.properties.viscosity as f32 * 0.05);
            self.speed += (target.value - self.value) * target_factor * elapsed;
        }
        self.value += self.speed * elapsed;
        if let Some(target) = self.target && !target.is_permanent {
            let target_sign_after = target.value > self.value;
            if target_sign_after != target_sign_before {
                self.target = None;
            }
        }
        if self.value < self.limit_down {
            self.value = self.limit_down;
        } else if self.value > self.limit_up {
            self.value = self.limit_up;
        }
        let friction_factor = if self.target.is_some() {
            48.0
        } else {
            3.0 / (1.0 + self.properties.inertion as f32 * 0.035)
        };
        self.speed /= 1.0 + elapsed * friction_factor;
        ((self.value - prev_value).abs() > 0.025, self.value)
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
        self.target = None;
        self.speed = 0.0;
    }
}