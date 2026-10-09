use crate::types::*;

pub(crate) fn zero(v: f64) -> f64 {
    if v == 0. { 0. } else { v }
}
pub(crate) fn finite(v: f64) -> Result<f64, Error> {
    if v.is_finite() {
        Ok(zero(v))
    } else {
        Err(err(
            ErrorCode::Overflow,
            "nonfinite arithmetic intermediate",
        ))
    }
}
pub(crate) fn range(v: f64, lo: f64, hi: f64, code: ErrorCode) -> Result<(), Error> {
    if !v.is_finite() || v < lo || v > hi {
        Err(err(code, "numeric bound violated"))
    } else {
        Ok(())
    }
}
pub(crate) fn point(v: [f64; 3]) -> Result<(), Error> {
    for x in v {
        range(x, -1e9, 1e9, ErrorCode::InvalidTransform)?;
    }
    Ok(())
}
pub(crate) fn clean_point(v: &mut [f64; 3]) {
    for x in v {
        *x = zero(*x);
    }
}
pub(crate) fn quaternion_canonical(q: Quaternion) -> bool {
    [q.w, q.x, q.y, q.z]
        .into_iter()
        .find(|v| *v != 0.)
        .is_some_and(|v| v > 0.)
}
pub fn normalize_quaternion(q: Quaternion) -> Result<Quaternion, Error> {
    let values = [q.x, q.y, q.z, q.w];
    if values.iter().any(|v| !v.is_finite()) {
        return Err(err(ErrorCode::InvalidTransform, "nonfinite quaternion"));
    }
    // Scaled norm avoids overflowing valid finite quaternion input.
    let max = values.iter().map(|v| v.abs()).fold(0., f64::max);
    if max == 0. {
        return Err(err(ErrorCode::InvalidTransform, "singular quaternion"));
    }
    let norm_scaled = values
        .iter()
        .map(|v| (v / max) * (v / max))
        .sum::<f64>()
        .sqrt();
    if max < 1e-12 / norm_scaled {
        return Err(err(
            ErrorCode::InvalidTransform,
            "quaternion norm below minimum",
        ));
    }
    let mut out = Quaternion {
        x: q.x / max / norm_scaled,
        y: q.y / max / norm_scaled,
        z: q.z / max / norm_scaled,
        w: q.w / max / norm_scaled,
    };
    if !quaternion_canonical(out) {
        out.x = -out.x;
        out.y = -out.y;
        out.z = -out.z;
        out.w = -out.w;
    }
    out.x = zero(out.x);
    out.y = zero(out.y);
    out.z = zero(out.z);
    out.w = zero(out.w);
    Ok(out)
}
pub(crate) fn valid_quaternion(q: Quaternion) -> Result<(), Error> {
    let v = [q.x, q.y, q.z, q.w];
    if v.iter().any(|x| !x.is_finite())
        || (v.iter().map(|x| x * x).sum::<f64>() - 1.).abs() > 1e-12
        || !quaternion_canonical(q)
    {
        return Err(err(
            ErrorCode::InvalidTransform,
            "quaternion is not canonical unit length",
        ));
    }
    Ok(())
}
fn rotate(q: Quaternion, v: [f64; 3], inverse: bool) -> Result<[f64; 3], Error> {
    valid_quaternion(q)?;
    let sign = if inverse { -1. } else { 1. };
    let [x, y, z] = [q.x * sign, q.y * sign, q.z * sign];
    let t = [
        2. * (y * v[2] - z * v[1]),
        2. * (z * v[0] - x * v[2]),
        2. * (x * v[1] - y * v[0]),
    ];
    let out = [
        v[0] + q.w * t[0] + y * t[2] - z * t[1],
        v[1] + q.w * t[1] + z * t[0] - x * t[2],
        v[2] + q.w * t[2] + x * t[1] - y * t[0],
    ];
    Ok([finite(out[0])?, finite(out[1])?, finite(out[2])?])
}
pub(crate) fn validate_view(t: &ViewTransform) -> Result<(), Error> {
    point(t.origin)?;
    valid_quaternion(t.rotation)?;
    for s in t.scale {
        range(s, 0.001, 1000., ErrorCode::InvalidTransform)?;
    }
    range(t.shear_xy, -8., 8., ErrorCode::InvalidTransform)
}
pub(crate) fn validate_pose(p: &CameraPose) -> Result<(), Error> {
    point(p.position)?;
    valid_quaternion(p.orientation)?;
    range(p.zoom, 0.001, 1000., ErrorCode::InvalidTransform)
}
pub(crate) fn lens(p: &Projection) -> [f64; 5] {
    match *p {
        Projection::Orthographic {
            half_height,
            near,
            far,
            focus_distance,
        } => [half_height, near, far, focus_distance, 0.],
        Projection::Perspective {
            half_height,
            near,
            far,
            focus_distance,
        } => [half_height, near, far, focus_distance, 1.],
        Projection::Blended {
            half_height,
            near,
            far,
            focus_distance,
            perspective_weight,
        } => [half_height, near, far, focus_distance, perspective_weight],
    }
}
pub(crate) fn validate_projection(p: &Projection) -> Result<(), Error> {
    let [h, n, f, d, b] = lens(p);
    range(h, 0.001, 1e9, ErrorCode::InvalidProjection)?;
    range(d, 0.001, 1e9, ErrorCode::InvalidProjection)?;
    range(n, 0.001, 1e8, ErrorCode::InvalidProjection)?;
    range(f, 0.001, 1e9, ErrorCode::InvalidProjection)?;
    range(b, 0., 1., ErrorCode::InvalidProjection)?;
    if f - n < 0.001 {
        return Err(err(
            ErrorCode::InvalidProjection,
            "invalid near/far interval",
        ));
    }
    Ok(())
}
pub(crate) fn validate_viewport(v: &Viewport) -> Result<(), Error> {
    range(v.x, -1e6, 1e6, ErrorCode::InvalidViewport)?;
    range(v.y, -1e6, 1e6, ErrorCode::InvalidViewport)?;
    range(v.width, 0.001, 16384., ErrorCode::InvalidViewport)?;
    range(v.height, 0.001, 16384., ErrorCode::InvalidViewport)
}
pub(crate) fn validate_target(t: &CameraTarget) -> Result<(), Error> {
    validate_view(&t.view_transform)?;
    validate_pose(&t.pose)?;
    validate_projection(&t.projection)
}
pub(crate) fn normalize_target(mut t: CameraTarget) -> Result<CameraTarget, Error> {
    t.pose.orientation = normalize_quaternion(t.pose.orientation)?;
    t.view_transform.rotation = normalize_quaternion(t.view_transform.rotation)?;
    clean_point(&mut t.pose.position);
    clean_point(&mut t.view_transform.origin);
    clean_point(&mut t.view_transform.scale);
    t.pose.zoom = zero(t.pose.zoom);
    t.view_transform.shear_xy = zero(t.view_transform.shear_xy);
    if let Projection::Blended {
        perspective_weight, ..
    } = &mut t.projection
    {
        *perspective_weight = zero(*perspective_weight);
    }
    validate_target(&t)?;
    Ok(t)
}
pub fn world_to_view(point: WorldPoint, transform: &ViewTransform) -> Result<ViewPoint, Error> {
    crate::math::point(point.0)?;
    validate_view(transform)?;
    let mut p = [0.; 3];
    for (i, item) in p.iter_mut().enumerate() {
        *item = finite((point.0[i] - transform.origin[i]) * transform.scale[i])?;
    }
    p[0] = finite(p[0] + transform.shear_xy * p[1])?;
    Ok(ViewPoint(rotate(transform.rotation, p, false)?))
}
pub fn view_to_world(point: ViewPoint, transform: &ViewTransform) -> Result<WorldPoint, Error> {
    for x in point.0 {
        finite(x)?;
    }
    validate_view(transform)?;
    let mut p = rotate(transform.rotation, point.0, true)?;
    p[0] = finite(p[0] - transform.shear_xy * p[1])?;
    for (i, item) in p.iter_mut().enumerate() {
        *item = finite(*item / transform.scale[i] + transform.origin[i])?;
    }
    crate::math::point(p)?;
    Ok(WorldPoint(p))
}
pub fn view_to_camera(point: ViewPoint, pose: &CameraPose) -> Result<CameraPoint, Error> {
    for x in point.0 {
        finite(x)?;
    }
    validate_pose(pose)?;
    let mut p = [0.; 3];
    for (i, item) in p.iter_mut().enumerate() {
        *item = finite(point.0[i] - pose.position[i])?;
    }
    Ok(CameraPoint(rotate(pose.orientation, p, true)?))
}
pub fn camera_to_view(point: CameraPoint, pose: &CameraPose) -> Result<ViewPoint, Error> {
    for x in point.0 {
        finite(x)?;
    }
    validate_pose(pose)?;
    let mut p = rotate(pose.orientation, point.0, false)?;
    for (i, item) in p.iter_mut().enumerate() {
        *item = finite(*item + pose.position[i])?;
    }
    Ok(ViewPoint(p))
}
pub fn project(
    point: WorldPoint,
    target: &CameraTarget,
    viewport: &Viewport,
) -> Result<ScreenPoint, Error> {
    validate_target(target)?;
    validate_viewport(viewport)?;
    let p = view_to_camera(world_to_view(point, &target.view_transform)?, &target.pose)?.0;
    let [height, near, far, focus, b] = lens(&target.projection);
    let d = finite(-p[2])?;
    if d < near || d > far {
        return Err(err(ErrorCode::OutsideDepth, "point outside near/far"));
    }
    let denominator = finite((1. - b) * focus + b * d)?;
    let h = finite(height / target.pose.zoom)?;
    let aspect = finite(viewport.width / viewport.height)?;
    let nx = finite(finite(p[0] * focus)? / finite(h * aspect * denominator)?)?;
    let ny = finite(finite(p[1] * focus)? / finite(h * denominator)?)?;
    Ok(ScreenPoint {
        x: finite(viewport.x + finite((nx + 1.) * viewport.width / 2.)?)?,
        y: finite(viewport.y + finite((1. - ny) * viewport.height / 2.)?)?,
        depth: finite((d - near) / (far - near))?,
    })
}
pub fn unproject(
    point: ScreenPoint,
    target: &CameraTarget,
    viewport: &Viewport,
) -> Result<WorldPoint, Error> {
    validate_target(target)?;
    validate_viewport(viewport)?;
    finite(point.x)?;
    finite(point.y)?;
    range(point.depth, 0., 1., ErrorCode::InvalidTransform)?;
    let [height, near, far, focus, b] = lens(&target.projection);
    let d = finite(near + point.depth * (far - near))?;
    let denominator = finite((1. - b) * focus + b * d)?;
    let h = finite(height / target.pose.zoom)?;
    let nx = finite((point.x - viewport.x) * 2. / viewport.width - 1.)?;
    let ny = finite(1. - (point.y - viewport.y) * 2. / viewport.height)?;
    let x = finite(nx * h * (viewport.width / viewport.height) * denominator / focus)?;
    let y = finite(ny * h * denominator / focus)?;
    view_to_world(
        camera_to_view(CameraPoint([x, y, -d]), &target.pose)?,
        &target.view_transform,
    )
}
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    // Roundoff must not escape the convex endpoint interval at accepted bounds.
    zero(((1. - t) * a + t * b).clamp(a.min(b), a.max(b)))
}
fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}
fn nlerp(a: Quaternion, mut b: Quaternion, t: f64) -> Result<Quaternion, Error> {
    if a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w < 0. {
        b.x = -b.x;
        b.y = -b.y;
        b.z = -b.z;
        b.w = -b.w;
    }
    normalize_quaternion(Quaternion {
        x: lerp(a.x, b.x, t),
        y: lerp(a.y, b.y, t),
        z: lerp(a.z, b.z, t),
        w: lerp(a.w, b.w, t),
    })
}
pub(crate) fn interpolate(
    a: &CameraTarget,
    b: &CameraTarget,
    elapsed: u32,
    duration: u32,
) -> Result<CameraTarget, Error> {
    if elapsed == 0 {
        return Ok(a.clone());
    }
    if elapsed == duration {
        return Ok(b.clone());
    }
    let u = f64::from(elapsed) / f64::from(duration);
    let t = u * u * (3. - 2. * u);
    let la = lens(&a.projection);
    let lb = lens(&b.projection);
    let mut l = [0.; 5];
    for i in 0..5 {
        l[i] = lerp(la[i], lb[i], t);
    }
    // Interpolate the positive depth interval rather than subtracting two
    // independently rounded large planes. Preserve the minimum interval even
    // when adding that small width to near rounds inward by one ULP.
    let width = lerp(la[2] - la[1], lb[2] - lb[1], t);
    l[2] = (l[1] + width).min(1e9);
    if l[2] - l[1] < 0.001 {
        l[2] = (l[1] + 0.001).next_up();
    }
    let out = CameraTarget {
        pose: CameraPose {
            position: lerp3(a.pose.position, b.pose.position, t),
            orientation: nlerp(a.pose.orientation, b.pose.orientation, t)?,
            zoom: lerp(a.pose.zoom, b.pose.zoom, t),
        },
        view_transform: ViewTransform {
            origin: lerp3(a.view_transform.origin, b.view_transform.origin, t),
            rotation: nlerp(a.view_transform.rotation, b.view_transform.rotation, t)?,
            scale: lerp3(a.view_transform.scale, b.view_transform.scale, t),
            shear_xy: lerp(a.view_transform.shear_xy, b.view_transform.shear_xy, t),
        },
        projection: Projection::Blended {
            half_height: l[0],
            near: l[1],
            far: l[2],
            focus_distance: l[3],
            perspective_weight: l[4],
        },
    };
    validate_target(&out)?;
    Ok(out)
}
