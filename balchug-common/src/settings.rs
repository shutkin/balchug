#[derive(Copy, Clone)]
pub struct InertiaProperties {
    pub viscosity: u8,
    pub inertion: u8,
}

impl Default for InertiaProperties {
    fn default() -> Self {
        Self {
            viscosity: 20,
            inertion: 50,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct BalchugSettings {
    pub background_color: [u8; 3],
    pub inertia_properties: InertiaProperties,
}
