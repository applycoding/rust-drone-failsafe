pub const MAX_FENCE_VERTS: usize = 16;
pub const HALF_RANGE_MS: u32 = 0x8000_0000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub n: f32,
    pub e: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub n: f32,
    pub e: f32,
    pub d: f32,
}

impl Vec3 {
    pub const fn alt_m(self) -> f32 {
        -self.d
    }

    pub fn horizontal_dist_home(self) -> f32 {
        libm::sqrtf(self.n * self.n + self.e * self.e)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stamped<T> {
    pub t_ms: u32,
    pub value: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpsFix {
    None,
    Fix2D,
    Fix3D,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandedState {
    Undefined,
    OnGround,
    InAir,
    Takeoff,
    Landing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionSample {
    pub t_ms: u32,
    pub pos: Vec3,
    pub vel: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsSample {
    pub t_ms: u32,
    pub fix: GpsFix,
    pub sats: Option<u8>,
    pub hdop: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryInputs {
    pub voltage_v: Option<Stamped<f32>>,
    pub percent: Option<Stamped<f32>>,
    pub remaining_wh: Option<Stamped<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Telemetry {
    pub now_ms: u32,
    pub position: Option<PositionSample>,
    pub gps: Option<GpsSample>,
    pub battery: BatteryInputs,
    pub gcs_heartbeat_ms: Option<u32>,
    pub armed: Option<Stamped<bool>>,
    pub landed: Option<Stamped<LandedState>>,
    pub rc_ok: Option<Stamped<bool>>,
    pub ekf_ok: Option<Stamped<bool>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Nominal,
    Warn,
    Hold,
    ReturnToLaunch,
    Land,
}

impl Action {
    pub const fn rank(self) -> u8 {
        match self {
            Action::Nominal => 0,
            Action::Warn => 1,
            Action::Hold => 2,
            Action::ReturnToLaunch => 3,
            Action::Land => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    None,
    ClockFault,
    InvalidInput,
    StaleBattery,
    GcsLinkLost,
    RcLost,
    GpsDegraded,
    GpsLost,
    EkfUnhealthy,
    PositionInvalid,
    FenceApproach,
    FenceBreach,
    AltitudeMax,
    AltitudeMin,
    BatteryWarn,
    BatteryLow,
    BatteryCritical,
    EnergyRtlMarginal,
    EnergyRtlInfeasible,
}

pub const N_REASONS: usize = 19;

impl Reason {
    pub const fn index(self) -> usize {
        match self {
            Reason::None => 0,
            Reason::ClockFault => 1,
            Reason::InvalidInput => 2,
            Reason::StaleBattery => 3,
            Reason::GcsLinkLost => 4,
            Reason::RcLost => 5,
            Reason::GpsDegraded => 6,
            Reason::GpsLost => 7,
            Reason::EkfUnhealthy => 8,
            Reason::PositionInvalid => 9,
            Reason::FenceApproach => 10,
            Reason::FenceBreach => 11,
            Reason::AltitudeMax => 12,
            Reason::AltitudeMin => 13,
            Reason::BatteryWarn => 14,
            Reason::BatteryLow => 15,
            Reason::BatteryCritical => 16,
            Reason::EnergyRtlMarginal => 17,
            Reason::EnergyRtlInfeasible => 18,
        }
    }

    pub const fn from_index(i: usize) -> Option<Reason> {
        match i {
            0 => Some(Reason::None),
            1 => Some(Reason::ClockFault),
            2 => Some(Reason::InvalidInput),
            3 => Some(Reason::StaleBattery),
            4 => Some(Reason::GcsLinkLost),
            5 => Some(Reason::RcLost),
            6 => Some(Reason::GpsDegraded),
            7 => Some(Reason::GpsLost),
            8 => Some(Reason::EkfUnhealthy),
            9 => Some(Reason::PositionInvalid),
            10 => Some(Reason::FenceApproach),
            11 => Some(Reason::FenceBreach),
            12 => Some(Reason::AltitudeMax),
            13 => Some(Reason::AltitudeMin),
            14 => Some(Reason::BatteryWarn),
            15 => Some(Reason::BatteryLow),
            16 => Some(Reason::BatteryCritical),
            17 => Some(Reason::EnergyRtlMarginal),
            18 => Some(Reason::EnergyRtlInfeasible),
            _ => None,
        }
    }

    /// Display-reason tie order among requests at the winning rank (higher wins).
    pub const fn display_priority(self) -> u8 {
        match self {
            Reason::BatteryCritical => 18,
            Reason::EnergyRtlInfeasible => 17,
            Reason::EkfUnhealthy => 16,
            Reason::GpsLost => 15,
            Reason::PositionInvalid => 14,
            Reason::FenceBreach => 13,
            Reason::AltitudeMax => 12,
            Reason::RcLost => 11,
            Reason::GcsLinkLost => 10,
            Reason::StaleBattery => 9,
            Reason::BatteryLow => 8,
            Reason::EnergyRtlMarginal => 7,
            Reason::ClockFault => 6,
            Reason::InvalidInput => 5,
            Reason::GpsDegraded => 4,
            Reason::FenceApproach => 3,
            Reason::AltitudeMin => 2,
            Reason::BatteryWarn => 1,
            Reason::None => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub reason: Reason,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActiveSet {
    pub reqs: [Option<Request>; N_REASONS],
}

impl ActiveSet {
    pub const fn new() -> Self {
        Self {
            reqs: [None; N_REASONS],
        }
    }

    pub fn set(&mut self, reason: Reason, action: Action) {
        let idx = reason.index();
        self.reqs[idx] = Some(Request { reason, action });
    }

    pub fn clear(&mut self, reason: Reason) {
        let idx = reason.index();
        self.reqs[idx] = None;
    }

    pub fn requested_action(&self) -> (Action, Reason) {
        let mut best_rank = 0u8;
        let mut best_reason = Reason::None;
        let mut best_action = Action::Nominal;
        for slot in self.reqs.iter().flatten() {
            let r = slot.action.rank();
            if r > best_rank {
                best_rank = r;
                best_action = slot.action;
                best_reason = slot.reason;
            } else if r == best_rank
                && slot.reason.display_priority() > best_reason.display_priority()
            {
                best_reason = slot.reason;
                best_action = slot.action;
            }
        }
        (best_action, best_reason)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyEstimate {
    pub rtl_wh: f32,
    pub land_wh: f32,
    pub reserve_wh: f32,
    pub remaining_wh: Option<f32>,
    pub margin_wh: Option<f32>,
    pub insufficient_for_landing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decision {
    pub action: Action,
    pub reason: Reason,
    pub requested: Action,
    pub active: ActiveSet,
    pub position_valid: bool,
    pub clock_fault: bool,
    pub energy: EnergyEstimate,
    pub fence_dist_m: Option<f32>,
    pub disarm_eligible: bool,
}
