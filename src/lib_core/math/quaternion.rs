use super::*;

type R = FixedNumber;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quaternion {
    // Scalar
    w: R,
    // Vector
    v: [R; 3],
}

// Derived from: https://github.com/MartinWeigel/Quaternion/blob/master/Quaternion.c

fn sin(f: R) -> R {
    FixedNumberLut::sin(f)
}
fn cos(f: R) -> R {
    FixedNumberLut::cos(f)
}
fn acos(f: R) -> R {
    unimplemented!();
}

impl Quaternion {
    pub fn new(w: R, v0: R, v1: R, v2: R) -> Self {
        Self { w, v: [v0, v1, v2] }
    }

    pub fn identity() -> Self {
        Self::new(1.into(), 0.into(), 0.into(), 0.into())
    }

    pub fn from_axis_angle(axis: Vec3, angle: R) -> Self {
        let w = cos(angle / 2.into());
        let c = sin(angle / 2.into());

        Self::new(w, c * axis.x, c * axis.y, c * axis.z)
    }

    pub fn to_axis_angle(&self) -> (R, Vec3) {
        let angle: R = FixedNumber::from_i32(2) * acos(self.w);
        let divider: R = (R::one() - self.w.sqrd()).sqrt();

        let output: Vec3 = {
            if divider != 0.into() {
                // Calculate the axis
                Vec3 {
                    x: self.v[0] / divider,
                    y: self.v[1] / divider,
                    z: self.v[2] / divider,
                }
            } else {
                // Arbitrary normalized axis
                (1, 0, 0).into()
            }
        };

        (angle, output)
    }

    pub fn from_x_rotation(angle: R) -> Self {
        Self::from_axis_angle((1, 0, 0).into(), angle)
    }

    pub fn from_y_rotation(angle: R) -> Self {
        Self::from_axis_angle((0, 1, 0).into(), angle)
    }

    pub fn from_z_rotation(angle: R) -> Self {
        Self::from_axis_angle((0, 0, 1).into(), angle)
    }

    /*
        void Quaternion_fromEulerZYX(double eulerZYX[3], Quaternion* output)
    {
        assert(output != NULL);
        // Based on https://en.wikipedia.org/wiki/Conversion_between_quaternions_and_Euler_angles
        double cy = cos(eulerZYX[2] * 0.5);
        double sy = sin(eulerZYX[2] * 0.5);
        double cr = cos(eulerZYX[0] * 0.5);
        double sr = sin(eulerZYX[0] * 0.5);
        double cp = cos(eulerZYX[1] * 0.5);
        double sp = sin(eulerZYX[1] * 0.5);

        output->w = cy * cr * cp + sy * sr * sp;
        output->v[0] = cy * sr * cp - sy * cr * sp;
        output->v[1] = cy * cr * sp + sy * sr * cp;
        output->v[2] = sy * cr * cp - cy * sr * sp;
    }

    void Quaternion_toEulerZYX(Quaternion* q, double output[3])
    {
        assert(output != NULL);

        // Roll (x-axis rotation)
        double sinr_cosp = +2.0 * (q->w * q->v[0] + q->v[1] * q->v[2]);
        double cosr_cosp = +1.0 - 2.0 * (q->v[0] * q->v[0] + q->v[1] * q->v[1]);
        output[0] = atan2(sinr_cosp, cosr_cosp);

        // Pitch (y-axis rotation)
        double sinp = +2.0 * (q->w * q->v[1] - q->v[2] * q->v[0]);
        if (fabs(sinp) >= 1)
            output[1] = copysign(M_PI / 2, sinp); // use 90 degrees if out of range
        else
            output[1] = asin(sinp);

        // Yaw (z-axis rotation)
        double siny_cosp = +2.0 * (q->w * q->v[2] + q->v[0] * q->v[1]);
        double cosy_cosp = +1.0 - 2.0 * (q->v[1] * q->v[1] + q->v[2] * q->v[2]);
        output[2] = atan2(siny_cosp, cosy_cosp);
    }

    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Quaternion_new_sets_as_expected() {
        let w: R = 3.into();
        let v0: R = 4.into();
        let v1: R = 5.into();
        let v2: R = 6.into();

        let q = Quaternion::new(w, v0, v1, v2);

        assert_eq!(w, q.w);
        assert_eq!(v0, q.v[0]);
        assert_eq!(v1, q.v[1]);
        assert_eq!(v2, q.v[2]);
    }

    #[test]
    fn Quaternion_identity_sets_as_expected() {
        let w: R = 1.into();
        let v0: R = 0.into();
        let v1: R = 0.into();
        let v2: R = 0.into();

        let q = Quaternion::identity();

        assert_eq!(w, q.w);
        assert_eq!(v0, q.v[0]);
        assert_eq!(v1, q.v[1]);
        assert_eq!(v2, q.v[2]);

        let q1 = Quaternion::new(w, v0, v1, v2);

        assert_eq!(q1, q);
    }
}
