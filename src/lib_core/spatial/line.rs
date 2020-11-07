use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
}

impl Line {
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        let norm = self.end - self.start;
        let t = (point - self.start).dot(norm) / norm.dot(norm);

        let modifier = { FixedNumber::min(FixedNumber::max(t, 0.into()), 1.into()) };

        return self.start + norm * modifier;
    }

    pub fn normalize(&self) -> Vec3 {
        (self.start - self.end).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Line_closest_point() {
        let line = Line {
            start: (0, 0, 0).into(),
            end: (0, 1, 0).into(),
        };

        let point: Vec3 = (0, -10, 0).into();

        let expected: Vec3 = (0, 0, 0).into();
        let actual = line.closest_point(point);
        assert_eq!(expected, actual);

        let line = Line {
            start: (1, 1, 1).into(),
            end: (2, 2, 2).into(),
        };

        let point: Vec3 = (0, 0, 0).into();

        let expected: Vec3 = (1, 1, 1).into();
        let actual = line.closest_point(point);
        assert_eq!(expected, actual);

        let line = Line {
            start: (1, 1, 1).into(),
            end: (2, 2, 2).into(),
        };

        let point: Vec3 = (0, 0, 0).into();

        let expected: Vec3 = (1, 1, 1).into();
        let actual = line.closest_point(point);
        assert_eq!(expected, actual);
    }
}
