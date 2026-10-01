//! Liquid "blob" surfaces: rounded rectangles that melt into each other.
//!
//! Every [`BlobRect`] is a signed distance field (a rounded box with per-corner radii);
//! shapes closer than the smoothing radius merge through a circular smooth-min, so a
//! panel sliding out of a bar grows out of it like a drop of liquid, and a panel that
//! moves squashes and stretches along its velocity ([`DeformSpring`]). This follows the
//! blob shader of the Caelestia shell, evaluated on the CPU: the merged field is sampled
//! on a grid around each cluster of nearby shapes, its zero contour is traced with
//! marching squares, and the outline is filled as an antialiased path.
//!
//! A shape can also melt into a screen edge ([`BlobRect::melt_into`]): where it meets
//! the edge the field grows a concave fillet against it, so a dock standing on the
//! bottom of the screen, or a bar hugging its sides, curves into the edge. The fillet is
//! part of the field, so it follows the shape as it moves, squashes or slides behind the
//! edge (it shrinks as the shape sinks and wells up as it rises).
//!
//! Typical use from a canvas paint callback:
//!
//! ```rust,no_run
//! use gpui_material::blob::{paint_blobs, BlobRect};
//! # fn paint(window: &mut gpui::Window, bar: gpui::Bounds<gpui::Pixels>, popup: gpui::Bounds<gpui::Pixels>) {
//! let rects = [BlobRect::new(bar).radius(0.0), BlobRect::new(popup).radius(28.0)];
//! paint_blobs(window, &rects, 20.0, gpui::rgb(0x1d1b20).into());
//! # }
//! ```

use std::collections::HashMap;

use gpui::{point, px, Bounds, PathBuilder, Pixels, Point, Rgba, Window};

/// A symmetric 2x2 deformation `[[m00, m01], [m01, m11]]` applied around a shape's
/// center: stretch along one axis, compress along the other.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Deform {
    /// Row 0, column 0.
    pub m00: f32,
    /// The off-diagonal term (the matrix is symmetric).
    pub m01: f32,
    /// Row 1, column 1.
    pub m11: f32,
}

impl Default for Deform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Deform {
    /// No deformation.
    pub const IDENTITY: Self = Self { m00: 1.0, m01: 0.0, m11: 1.0 };

    fn inverse(self) -> (f32, f32, f32) {
        let det = self.m00 * self.m11 - self.m01 * self.m01;
        if det.abs() < 1e-6 {
            return (1.0, 0.0, 1.0);
        }
        (self.m11 / det, -self.m01 / det, self.m00 / det)
    }

    /// Smallest eigenvalue, the SDF scale correction for the deformed shape.
    fn min_eigen(self) -> f32 {
        let tr = self.m00 + self.m11;
        let d = ((self.m00 - self.m11).powi(2) / 4.0 + self.m01 * self.m01).sqrt();
        tr / 2.0 - d
    }

    fn max_eigen(self) -> f32 {
        let tr = self.m00 + self.m11;
        let d = ((self.m00 - self.m11).powi(2) / 4.0 + self.m01 * self.m01).sqrt();
        tr / 2.0 + d
    }

    /// True when the deformation is visually imperceptible.
    pub fn is_identity(self) -> bool {
        (self.m00 - 1.0).abs() + self.m01.abs() + (self.m11 - 1.0).abs() < 0.004
    }
}

/// Velocity driven squash and stretch: an underdamped spring on each matrix component,
/// pulled towards a stretch along the direction of motion.
#[derive(Clone, Copy, Debug)]
pub struct DeformSpring {
    /// Current deformation.
    pub value: Deform,
    velocity: (f32, f32, f32),
    prev_center: Option<Point<Pixels>>,
    active: bool,
    /// Spring stiffness (default 200).
    pub stiffness: f32,
    /// Damping (default 16).
    pub damping: f32,
    /// Stretch per px/s of speed (default 0.0005 * 0.15, a panel-ish amount).
    pub deform_scale: f32,
}

impl Default for DeformSpring {
    fn default() -> Self {
        Self {
            value: Deform::IDENTITY,
            velocity: (0.0, 0.0, 0.0),
            prev_center: None,
            active: false,
            stiffness: 200.0,
            damping: 16.0,
            deform_scale: 0.000_075,
        }
    }
}

impl DeformSpring {
    /// A spring with a given deform scale (stretch per px/s).
    #[must_use]
    pub fn with_scale(deform_scale: f32) -> Self {
        Self { deform_scale, ..Self::default() }
    }

    /// True while the shape is still wobbling and needs frames.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advances the spring given the shape's current center and the frame time.
    pub fn step(&mut self, center: Point<Pixels>, dt: f32) -> Deform {
        let Some(prev) = self.prev_center.replace(center) else { return self.value };
        if dt < 0.001 {
            return self.value;
        }
        // long frames still make progress, clamped so the spring stays stable
        let dt = dt.min(0.05);
        let vx = f32::from(center.x - prev.x) / dt;
        let vy = f32::from(center.y - prev.y) / dt;
        let speed = (vx * vx + vy * vy).sqrt();
        if !self.active {
            if speed < 5.0 {
                return self.value;
            }
            self.active = true;
        }
        const MAX_STRETCH: f32 = 0.35;
        let (mut t00, mut t01, mut t11) = (1.0, 0.0, 1.0);
        if speed > 5.0 {
            let stretch = 1.0 + (speed * self.deform_scale).min(MAX_STRETCH);
            let compress = 1.0 / stretch;
            let (c, s) = (vx / speed, vy / speed);
            t00 = stretch * c * c + compress * s * s;
            t01 = (stretch - compress) * c * s;
            t11 = stretch * s * s + compress * c * c;
        }
        // implicit damping keeps the spring stable on slow frames
        let inv_damp = 1.0 / (1.0 + self.damping * dt);
        let k = self.stiffness;
        let v = &mut self.velocity;
        v.0 = (v.0 - k * (self.value.m00 - t00) * dt) * inv_damp;
        v.1 = (v.1 - k * (self.value.m01 - t01) * dt) * inv_damp;
        v.2 = (v.2 - k * (self.value.m11 - t11) * dt) * inv_damp;
        self.value.m00 += v.0 * dt;
        self.value.m01 += v.1 * dt;
        self.value.m11 += v.2 * dt;
        let vel = v.0.abs() + v.1.abs() + v.2.abs();
        if self.value.is_identity() && vel < 0.05 && speed < 5.0 {
            *self = Self { stiffness: self.stiffness, damping: self.damping, deform_scale: self.deform_scale, prev_center: Some(center), ..Self::default() };
        }
        self.value
    }
}

/// A screen edge a shape can melt into, at a coordinate along its axis: `Bottom(1080.0)`
/// is the line `y = 1080` with the screen above it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    /// The line `x = at`, screen to its right.
    Left(f32),
    /// The line `y = at`, screen below it.
    Top(f32),
    /// The line `x = at`, screen to its left.
    Right(f32),
    /// The line `y = at`, screen above it.
    Bottom(f32),
}

impl Edge {
    /// Distance from the edge into the screen; negative past it.
    fn inside(self, x: f32, y: f32) -> f32 {
        match self {
            Edge::Left(a) => x - a,
            Edge::Top(a) => y - a,
            Edge::Right(a) => a - x,
            Edge::Bottom(a) => a - y,
        }
    }
}

/// One shape of a blob group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlobRect {
    /// Layout bounds (before deformation).
    pub bounds: Bounds<Pixels>,
    /// Corner radii: top left, top right, bottom right, bottom left.
    pub radii: [f32; 4],
    /// Squash and stretch around the center.
    pub deform: Deform,
    /// Bit `i` set: don't smooth-merge with shape `i` of the same slice (shapes only
    /// touch with a hard union).
    pub exclude: u64,
    /// Edges the shape melts into, each with its fillet radius.
    pub melt: [Option<(Edge, f32)>; 4],
}

impl BlobRect {
    /// A shape with square corners.
    #[must_use]
    pub fn new(bounds: Bounds<Pixels>) -> Self {
        Self { bounds, radii: [0.0; 4], deform: Deform::IDENTITY, exclude: 0, melt: [None; 4] }
    }

    /// Same radius on every corner.
    #[must_use]
    pub fn radius(mut self, r: f32) -> Self {
        self.radii = [r; 4];
        self
    }

    /// Per-corner radii: top left, top right, bottom right, bottom left.
    #[must_use]
    pub fn radii(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.radii = [tl, tr, br, bl];
        self
    }

    /// Applies a deformation.
    #[must_use]
    pub fn deform(mut self, deform: Deform) -> Self {
        self.deform = deform;
        self
    }

    /// Hard union (no smooth merge) with the shape at `index`.
    #[must_use]
    pub fn exclude(mut self, index: usize) -> Self {
        self.exclude |= 1 << index.min(63);
        self
    }

    /// Melts the shape into `edge`: where it is within `radius` of the edge, a concave
    /// fillet of that radius joins the two, cut off at the edge itself. Up to four edges.
    #[must_use]
    pub fn melt_into(mut self, edge: Edge, radius: f32) -> Self {
        if let Some(slot) = self.melt.iter_mut().find(|m| m.is_none()) {
            *slot = Some((edge, radius.max(0.0)));
        }
        self
    }

    fn prepared(&self) -> Prepared {
        let (x, y) = (f32::from(self.bounds.origin.x), f32::from(self.bounds.origin.y));
        let (w, h) = (f32::from(self.bounds.size.width), f32::from(self.bounds.size.height));
        let max_r = w.min(h) / 2.0;
        let r = self.radii.map(|r| r.clamp(0.0, max_r.max(0.0)));
        let stretch = self.deform.max_eigen().max(1.0);
        Prepared {
            cx: x + w / 2.0,
            cy: y + h / 2.0,
            hw: w / 2.0,
            hh: h / 2.0,
            // tl, tr, br, bl
            r,
            inv: self.deform.inverse(),
            min_eig: self.deform.min_eigen().max(0.01),
            ext_x: w / 2.0 * stretch,
            ext_y: h / 2.0 * stretch,
            exclude: self.exclude,
            melt: self.melt,
            reach: self.melt.iter().flatten().map(|m| m.1).fold(0.0, f32::max),
            empty: w < 0.5 || h < 0.5,
        }
    }
}

#[derive(Clone, Copy)]
struct Prepared {
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
    r: [f32; 4],
    inv: (f32, f32, f32),
    min_eig: f32,
    ext_x: f32,
    ext_y: f32,
    exclude: u64,
    melt: [Option<(Edge, f32)>; 4],
    /// Largest melt radius: how far past the shape its fillets reach.
    reach: f32,
    empty: bool,
}

impl Prepared {
    /// Outline of the (deformed) rounded box, grown by `iso`.
    fn outline(&self, iso: f32) -> Vec<Point<Pixels>> {
        let grow = iso / self.min_eig;
        let (hw, hh) = (self.hw + grow, self.hh + grow);
        let (i00, i01, i11) = self.inv;
        // forward deform = inverse of the stored inverse
        let det = i00 * i11 - i01 * i01;
        let (m00, m01, m11) = if det.abs() < 1e-6 { (1.0, 0.0, 1.0) } else { (i11 / det, -i01 / det, i00 / det) };
        let mut pts = Vec::new();
        // corners clockwise from top left: center offset signs and start angle
        let corners = [(-1.0f32, -1.0f32, std::f32::consts::PI, self.r[0]), (1.0, -1.0, 1.5 * std::f32::consts::PI, self.r[1]), (1.0, 1.0, 0.0, self.r[2]), (-1.0, 1.0, 0.5 * std::f32::consts::PI, self.r[3])];
        for (sx, sy, a0, r) in corners {
            let r = (r + grow).max(0.0).min(hw.min(hh));
            let (ccx, ccy) = (sx * (hw - r), sy * (hh - r));
            let n = if r < 0.5 { 1 } else { ((r / 2.0).ceil() as usize).clamp(4, 24) };
            for k in 0..=n {
                let a = a0 + std::f32::consts::FRAC_PI_2 * k as f32 / n as f32;
                let (qx, qy) = (ccx + r * a.cos(), ccy + r * a.sin());
                pts.push(point(px(self.cx + m00 * qx + m01 * qy), px(self.cy + m01 * qx + m11 * qy)));
            }
        }
        pts
    }

    fn sdf(&self, px_: f32, py: f32) -> f32 {
        let (dx, dy) = (px_ - self.cx, py - self.cy);
        let (i00, i01, i11) = self.inv;
        let (qx, qy) = (i00 * dx + i01 * dy, i01 * dx + i11 * dy);
        // radius of the quadrant the point is in
        let r = match (qx > 0.0, qy > 0.0) {
            (false, false) => self.r[0],
            (true, false) => self.r[1],
            (true, true) => self.r[2],
            (false, true) => self.r[3],
        };
        let ax = qx.abs() - self.hw + r;
        let ay = qy.abs() - self.hh + r;
        let outside = (ax.max(0.0).powi(2) + ay.max(0.0).powi(2)).sqrt();
        (ax.max(ay).min(0.0) + outside - r) * self.min_eig
    }
}

/// Circular smooth minimum: a fillet of radius `k` between the two surfaces.
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let ma = (k - a).max(0.0);
    let mb = (k - b).max(0.0);
    k.max(a.min(b)) - (ma * ma + mb * mb).sqrt()
}

fn merged_sdf(shapes: &[(usize, Prepared)], x: f32, y: f32, k: f32) -> f32 {
    let mut d = [f32::MAX; 64];
    let mut merged = f32::MAX;
    for (n, (_, s)) in shapes.iter().enumerate().take(64) {
        let reach = k.max(s.reach) * 1.5 + 4.0;
        if (x - s.cx).abs() > s.ext_x + reach || (y - s.cy).abs() > s.ext_y + reach {
            // far away: only matters as "outside"
            d[n] = f32::MAX;
            continue;
        }
        d[n] = s.sdf(x, y);
        merged = merged.min(d[n]);
        // fillets into the edges it melts into, only on the screen side of each edge
        for (edge, r) in s.melt.iter().flatten() {
            if d[n] >= *r {
                continue;
            }
            let de = edge.inside(x, y);
            if (0.0..*r).contains(&de) {
                merged = merged.min(smin(d[n], de, *r));
            }
        }
    }
    for i in 0..shapes.len().min(64) {
        if d[i] >= k {
            continue;
        }
        for j in i + 1..shapes.len().min(64) {
            if d[j] >= k {
                continue;
            }
            let (oi, oj) = (shapes[i].0, shapes[j].0);
            if shapes[i].1.exclude & (1 << oj.min(63)) != 0 || shapes[j].1.exclude & (1 << oi.min(63)) != 0 {
                continue;
            }
            merged = merged.min(smin(d[i], d[j], k));
        }
    }
    if merged == f32::MAX {
        1e6
    } else {
        merged
    }
}

/// Outlines of the merged shapes where the field equals `iso` (0 for the surface,
/// positive values for grown copies, e.g. for soft shadows). `step` is the grid size in
/// pixels; 2 gives smooth curves.
pub fn blob_contours(rects: &[BlobRect], smoothing: f32, step: f32, iso: f32) -> Vec<Vec<Point<Pixels>>> {
    let k = smoothing.max(0.5);
    let step = step.max(0.5);
    let prepared: Vec<(usize, Prepared)> = rects.iter().enumerate().map(|(i, r)| (i, r.prepared())).filter(|(_, p)| !p.empty).collect();
    let pad = |p: &Prepared| k.max(p.reach) * 1.5 + iso.max(0.0) + step * 2.0;
    // group shapes whose padded boxes touch, each cluster gets its own grid
    let boxes: Vec<(f32, f32, f32, f32)> = prepared
        .iter()
        .map(|(_, p)| {
            let pad = pad(p);
            (p.cx - p.ext_x - pad, p.cy - p.ext_y - pad, p.cx + p.ext_x + pad, p.cy + p.ext_y + pad)
        })
        .collect();
    let mut cluster = (0..prepared.len()).collect::<Vec<_>>();
    fn find(c: &mut [usize], i: usize) -> usize {
        let mut i = i;
        while c[i] != i {
            c[i] = c[c[i]];
            i = c[i];
        }
        i
    }
    for i in 0..boxes.len() {
        for j in i + 1..boxes.len() {
            let (a, b) = (boxes[i], boxes[j]);
            if a.0 <= b.2 && b.0 <= a.2 && a.1 <= b.3 && b.1 <= a.3 {
                let (ri, rj) = (find(&mut cluster, i), find(&mut cluster, j));
                cluster[ri] = rj;
            }
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..prepared.len() {
        let root = find(&mut cluster, i);
        groups.entry(root).or_default().push(i);
    }
    let mut out = Vec::new();
    for members in groups.values() {
        if let [only] = members.as_slice() {
            // nothing to merge with: the outline is exact and needs no grid
            if prepared[*only].1.reach <= 0.0 {
                out.push(prepared[*only].1.outline(iso));
                continue;
            }
        }
        let shapes: Vec<(usize, Prepared)> = members.iter().map(|&i| prepared[i]).collect();
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &i in members {
            let b = boxes[i];
            x0 = x0.min(b.0);
            y0 = y0.min(b.1);
            x1 = x1.max(b.2);
            y1 = y1.max(b.3);
        }
        x0 = (x0 / step).floor() * step;
        y0 = (y0 / step).floor() * step;
        out.extend(march(&shapes, k, iso, x0, y0, ((x1 - x0) / step).ceil() as usize + 1, ((y1 - y0) / step).ceil() as usize + 1, step));
    }
    out
}

/// The merged SDF on a `vw x vh` vertex grid. It is sampled on a coarse grid first and
/// only evaluated at full resolution in coarse cells the surface can pass through (the
/// field is 1-Lipschitz, so a cell whose corners are all farther than its diagonal from
/// the surface can't contain it); elsewhere the coarse sign is enough.
#[allow(clippy::too_many_arguments)]
fn sample_field(shapes: &[(usize, Prepared)], k: f32, iso: f32, x0: f32, y0: f32, vw: usize, vh: usize, step: f32) -> Vec<f32> {
    const C: usize = 4;
    let f = |i: usize, j: usize| merged_sdf(shapes, x0 + i as f32 * step, y0 + j as f32 * step, k) - iso;
    let (cw, ch) = ((vw - 1).div_ceil(C) + 1, (vh - 1).div_ceil(C) + 1);
    let coarse: Vec<f32> = (0..ch * cw).map(|n| f(((n % cw) * C).min(vw - 1), ((n / cw) * C).min(vh - 1))).collect();
    let safe = C as f32 * step * std::f32::consts::SQRT_2 * 1.25;
    let mut field = vec![f32::NAN; vw * vh];
    for cj in 0..ch - 1 {
        for ci in 0..cw - 1 {
            let corners = [coarse[cj * cw + ci], coarse[cj * cw + ci + 1], coarse[(cj + 1) * cw + ci], coarse[(cj + 1) * cw + ci + 1]];
            if corners.iter().all(|v| v.abs() > safe) {
                continue;
            }
            for j in cj * C..=((cj + 1) * C).min(vh - 1) {
                for i in ci * C..=((ci + 1) * C).min(vw - 1) {
                    let n = j * vw + i;
                    if field[n].is_nan() {
                        field[n] = f(i, j);
                    }
                }
            }
        }
    }
    for j in 0..vh {
        for i in 0..vw {
            let n = j * vw + i;
            if field[n].is_nan() {
                let (ci, cj) = (((i + C / 2) / C).min(cw - 1), ((j + C / 2) / C).min(ch - 1));
                field[n] = coarse[cj * cw + ci];
            }
        }
    }
    field
}

/// Marching squares over a `w x h` cell grid starting at `(x0, y0)`.
#[allow(clippy::too_many_arguments)]
fn march(shapes: &[(usize, Prepared)], k: f32, iso: f32, x0: f32, y0: f32, w: usize, h: usize, step: f32) -> Vec<Vec<Point<Pixels>>> {
    let (vw, vh) = (w + 1, h + 1);
    let field = sample_field(shapes, k, iso, x0, y0, vw, vh, step);
    let v = |i: usize, j: usize| field[j * vw + i];
    let h_id = |i: usize, j: usize| j * w + i;
    let v_id = |i: usize, j: usize| vw * vh + j * vw + i;
    let mut points: HashMap<usize, Point<Pixels>> = HashMap::new();
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    let lerp = |a: f32, b: f32| (a / (a - b)).clamp(0.0, 1.0);
    for j in 0..h {
        for i in 0..w {
            let (a, b, c, d) = (v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1));
            let case = (a < 0.0) as u8 | ((b < 0.0) as u8) << 1 | ((c < 0.0) as u8) << 2 | ((d < 0.0) as u8) << 3;
            if case == 0 || case == 15 {
                continue;
            }
            // edges: 0 top, 1 right, 2 bottom, 3 left
            let edge = |e: u8| -> usize {
                match e {
                    0 => h_id(i, j),
                    1 => v_id(i + 1, j),
                    2 => h_id(i, j + 1),
                    _ => v_id(i, j),
                }
            };
            let (fx, fy) = (x0 + i as f32 * step, y0 + j as f32 * step);
            let mut pt = |e: u8| {
                let id = edge(e);
                points.entry(id).or_insert_with(|| {
                    let (px_, py) = match e {
                        0 => (fx + lerp(a, b) * step, fy),
                        1 => (fx + step, fy + lerp(b, c) * step),
                        2 => (fx + lerp(d, c) * step, fy + step),
                        _ => (fx, fy + lerp(a, d) * step),
                    };
                    point(px(px_), px(py))
                });
                id
            };
            let center_in = (a + b + c + d) / 4.0 < 0.0;
            let segs: &[(u8, u8)] = match case {
                1 | 14 => &[(3, 0)],
                2 | 13 => &[(0, 1)],
                3 | 12 => &[(3, 1)],
                4 | 11 => &[(1, 2)],
                6 | 9 => &[(0, 2)],
                7 | 8 => &[(3, 2)],
                5 => {
                    if center_in {
                        &[(0, 1), (2, 3)]
                    } else {
                        &[(3, 0), (1, 2)]
                    }
                }
                _ => {
                    // 10
                    if center_in {
                        &[(3, 0), (1, 2)]
                    } else {
                        &[(0, 1), (2, 3)]
                    }
                }
            };
            for &(ea, eb) in segs {
                let (ia, ib) = (pt(ea), pt(eb));
                adj.entry(ia).or_default().push(ib);
                adj.entry(ib).or_default().push(ia);
            }
        }
    }
    // walk the segment graph into closed loops
    let mut loops = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let keys: Vec<usize> = adj.keys().copied().collect();
    for start in keys {
        if visited.contains(&start) {
            continue;
        }
        let mut path = vec![start];
        visited.insert(start);
        let mut prev = usize::MAX;
        let mut cur = start;
        loop {
            let Some(next) = adj.get(&cur).and_then(|n| n.iter().copied().find(|&x| x != prev && !visited.contains(&x)).or_else(|| n.iter().copied().find(|&x| x == start && prev != usize::MAX && x != prev))) else { break };
            if next == start {
                break;
            }
            visited.insert(next);
            path.push(next);
            prev = cur;
            cur = next;
        }
        if path.len() >= 3 {
            loops.push(path.into_iter().filter_map(|id| points.get(&id).copied()).collect());
        }
    }
    loops
}

/// Fills the merged outline of `rects`. `smoothing` is the fillet radius where shapes
/// meet (20 is a good default).
pub fn paint_blobs(window: &mut Window, rects: &[BlobRect], smoothing: f32, color: Rgba) {
    paint_contours(window, &blob_contours(rects, smoothing, 2.0, 0.0), color);
}

/// Soft shadow under a blob group: a few grown, translucent copies of the outline.
pub fn paint_blob_shadow(window: &mut Window, rects: &[BlobRect], smoothing: f32, color: Rgba, spread: f32, offset_y: f32) {
    let layers = 4;
    let shifted: Vec<BlobRect> = rects
        .iter()
        .map(|r| {
            let mut r = *r;
            r.bounds.origin.y += px(offset_y);
            r
        })
        .collect();
    for n in (1..=layers).rev() {
        let iso = spread * n as f32 / layers as f32;
        let alpha = color.a / layers as f32;
        paint_contours(window, &blob_contours(&shifted, smoothing, 3.0, iso), Rgba { a: alpha, ..color });
    }
}

/// Fills precomputed outlines, see [`blob_contours`].
pub fn paint_contours(window: &mut Window, contours: &[Vec<Point<Pixels>>], color: Rgba) {
    if contours.is_empty() {
        return;
    }
    let mut b = PathBuilder::fill();
    for c in contours {
        b.add_polygon(c, true);
    }
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> BlobRect {
        BlobRect::new(Bounds::new(point(px(x), px(y)), size(px(w), px(h))))
    }

    #[test]
    fn sdf_matches_box() {
        let p = rect(0.0, 0.0, 100.0, 50.0).prepared();
        assert!(p.sdf(50.0, 25.0) < 0.0);
        assert!((p.sdf(110.0, 25.0) - 10.0).abs() < 0.01);
        assert!((p.sdf(50.0, -5.0) - 5.0).abs() < 0.01);
    }

    #[test]
    fn rounded_corner_distance() {
        let p = rect(0.0, 0.0, 100.0, 100.0).radius(20.0).prepared();
        // the corner arc is centered at (20, 20)
        let d = p.sdf(0.0, 0.0);
        assert!((d - (20.0 * 2f32.sqrt() - 20.0)).abs() < 0.01);
    }

    #[test]
    fn separate_shapes_make_separate_loops() {
        let c = blob_contours(&[rect(0.0, 0.0, 40.0, 40.0), rect(200.0, 0.0, 40.0, 40.0)], 10.0, 2.0, 0.0);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn close_shapes_merge_into_one() {
        // 8px apart, within the 20px smoothing radius: one outline
        let c = blob_contours(&[rect(0.0, 0.0, 100.0, 40.0), rect(30.0, 48.0, 40.0, 40.0)], 20.0, 2.0, 0.0);
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn excluded_shapes_do_not_bridge() {
        let c = blob_contours(&[rect(0.0, 0.0, 100.0, 40.0).exclude(1), rect(30.0, 48.0, 40.0, 40.0)], 20.0, 2.0, 0.0);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn melting_into_an_edge_fills_the_concave_corner() {
        // a 100x40 shape standing on y = 100, melting into it with 20px fillets
        let shape = rect(50.0, 60.0, 100.0, 40.0).melt_into(Edge::Bottom(100.0), 20.0);
        let s = shape.prepared();
        let field = |x: f32, y: f32| merged_sdf(&[(0, s)], x, y, 4.0);
        // just outside the corner, inside the fillet: filled
        assert!(field(46.0, 98.0) < 0.0);
        // beyond the fillet radius along the edge: empty
        assert!(field(25.0, 98.0) > 0.0);
        // past the edge: only the shape itself
        assert!(field(46.0, 101.0) > 0.0);
        // and it traces as one outline wider than the shape
        let c = blob_contours(&[shape], 4.0, 1.0, 0.0);
        assert_eq!(c.len(), 1);
        let min_x = c[0].iter().map(|p| f32::from(p.x)).fold(f32::MAX, f32::min);
        assert!(min_x < 40.0, "fillet should reach out past the shape, got {min_x}");
    }

    #[test]
    fn a_shape_away_from_the_edge_is_untouched() {
        let shape = rect(50.0, 20.0, 100.0, 40.0).melt_into(Edge::Bottom(100.0), 20.0);
        let s = shape.prepared();
        assert!(merged_sdf(&[(0, s)], 46.0, 98.0, 4.0) > 0.0);
    }

    #[test]
    fn spring_settles() {
        let mut s = DeformSpring::with_scale(0.0005);
        let mut t = 0.0;
        for i in 0..20 {
            s.step(point(px(i as f32 * 20.0), px(0.0)), 0.016);
            t += 0.016;
        }
        assert!(!s.value.is_identity(), "moving shape stretches");
        for _ in 0..400 {
            s.step(point(px(400.0), px(0.0)), 0.016);
        }
        assert!(s.value.is_identity() && !s.is_active(), "settles after stopping ({t})");
    }

    #[test]
    fn lone_outline_lies_on_the_surface() {
        let r = rect(10.0, 20.0, 120.0, 60.0).radii(4.0, 12.0, 20.0, 0.0).deform(Deform { m00: 1.2, m01: 0.1, m11: 0.85 });
        let p = r.prepared();
        for pt in p.outline(0.0) {
            assert!(p.sdf(f32::from(pt.x), f32::from(pt.y)).abs() < 0.05);
        }
    }

    #[test]
    fn coarse_sampling_matches_full_sampling() {
        let shapes: Vec<(usize, Prepared)> = [rect(0.0, 0.0, 200.0, 40.0).radius(20.0), rect(60.0, 38.0, 90.0, 120.0).radius(16.0)]
            .iter()
            .enumerate()
            .map(|(i, r)| (i, r.prepared()))
            .collect();
        let (vw, vh, step) = (140, 110, 2.0);
        let field = sample_field(&shapes, 18.0, 0.0, -30.0, -30.0, vw, vh, step);
        for j in 0..vh {
            for i in 0..vw {
                let exact = merged_sdf(&shapes, -30.0 + i as f32 * step, -30.0 + j as f32 * step, 18.0);
                assert_eq!(field[j * vw + i] < 0.0, exact < 0.0, "sign differs at {i},{j}");
            }
        }
    }
}
