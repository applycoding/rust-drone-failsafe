use crate::types::{MAX_FENCE_VERTS, Point2};
use libm::fabsf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FenceError {
    TooFewVertices,
    TooManyVertices,
    NonFinite,
    OutOfEnvelope,
    DuplicateVertex,
    CollinearEdge,
    NotConvex,
    AreaTooSmall,
    BadAltitudes,
}

#[derive(Clone, Copy, Debug)]
pub struct ConvexPolygon {
    verts: [Point2; MAX_FENCE_VERTS],
    len: u8,
}

impl ConvexPolygon {
    pub fn new(pts: &[Point2]) -> Result<Self, FenceError> {
        if pts.len() < 3 {
            return Err(FenceError::TooFewVertices);
        }
        if pts.len() > MAX_FENCE_VERTS {
            return Err(FenceError::TooManyVertices);
        }
        for p in pts {
            if !p.n.is_finite() || !p.e.is_finite() {
                return Err(FenceError::NonFinite);
            }
            if fabsf(p.n) > 10_000.0 || fabsf(p.e) > 10_000.0 {
                return Err(FenceError::OutOfEnvelope);
            }
        }
        for i in 0..pts.len() {
            for j in (i + 1)..pts.len() {
                if pts[i].n == pts[j].n && pts[i].e == pts[j].e {
                    return Err(FenceError::DuplicateVertex);
                }
            }
        }
        let (verts, len) = normalize_ccw(pts)?;
        Ok(Self { len, verts })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn vertex(&self, i: usize) -> Point2 {
        self.verts[i]
    }

    pub fn contains(&self, p: Point2) -> bool {
        self.signed_distance(p) >= 0.0
    }

    /// Minimum inward distance to edges; >= 0 inside (boundary counts as inside).
    pub fn signed_distance(&self, p: Point2) -> f32 {
        let n = self.len();
        if n < 3 {
            return -1.0;
        }
        let mut min_sd = f32::INFINITY;
        for i in 0..n {
            let a = self.verts[i];
            let b = self.verts[(i + 1) % n];
            let sd = edge_signed_distance(a, b, p);
            if sd < min_sd {
                min_sd = sd;
            }
        }
        min_sd
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Geofence {
    pub poly: ConvexPolygon,
    pub max_alt_m: f32,
    pub min_alt_m: f32,
}

impl Geofence {
    pub fn new(poly: ConvexPolygon, min_alt_m: f32, max_alt_m: f32) -> Result<Self, FenceError> {
        if !min_alt_m.is_finite() || !max_alt_m.is_finite() || min_alt_m >= max_alt_m {
            return Err(FenceError::BadAltitudes);
        }
        Ok(Self {
            poly,
            max_alt_m,
            min_alt_m,
        })
    }
}

fn cross2(o: Point2, a: Point2, b: Point2) -> f32 {
    (a.n - o.n) * (b.e - o.e) - (a.e - o.e) * (b.n - o.n)
}

fn edge_signed_distance(a: Point2, b: Point2, p: Point2) -> f32 {
    let ab_n = b.n - a.n;
    let ab_e = b.e - a.e;
    let len = libm::sqrtf(ab_n * ab_n + ab_e * ab_e);
    if len < 1e-6 {
        return -1.0;
    }
    let ap_n = p.n - a.n;
    let ap_e = p.e - a.e;
    let cross = ab_n * ap_e - ab_e * ap_n;
    cross / len
}

fn polygon_area(pts: &[Point2]) -> f32 {
    let mut area = 0.0f32;
    let n = pts.len();
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        area += a.n * b.e - b.n * a.e;
    }
    area * 0.5
}

fn normalize_ccw(pts: &[Point2]) -> Result<([Point2; MAX_FENCE_VERTS], u8), FenceError> {
    let n = pts.len();
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let c = pts[(i + 2) % n];
        let ab_n = b.n - a.n;
        let ab_e = b.e - a.e;
        let bc_n = c.n - b.n;
        let bc_e = c.e - b.e;
        let cross_ab_bc = ab_n * bc_e - ab_e * bc_n;
        if fabsf(cross_ab_bc) < 1e-4 {
            return Err(FenceError::CollinearEdge);
        }
    }

    let mut pos = 0i32;
    let mut neg = 0i32;
    for i in 0..n {
        let c = cross2(pts[i], pts[(i + 1) % n], pts[(i + 2) % n]);
        if c > 0.0 {
            pos += 1;
        } else if c < 0.0 {
            neg += 1;
        }
    }
    if pos > 0 && neg > 0 {
        return Err(FenceError::NotConvex);
    }
    let area = polygon_area(pts);
    if fabsf(area) < 100.0 {
        return Err(FenceError::AreaTooSmall);
    }

    let ccw = area > 0.0;
    let mut verts = [Point2 { n: 0.0, e: 0.0 }; MAX_FENCE_VERTS];
    let len = n as u8;
    if ccw {
        for (i, p) in pts.iter().enumerate() {
            verts[i] = *p;
        }
    } else {
        for (i, p) in pts.iter().rev().enumerate() {
            verts[i] = *p;
        }
    }
    Ok((verts, len))
}
