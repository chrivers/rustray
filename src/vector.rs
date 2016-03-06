mod vector
{
    use std::ops::{Add, Sub};

    struct Vector<Float>
    {
        x: Float,
        y: Float,
        z: Float,
    }

    impl<Float: Add<Output=Float>> Add for Vector<Float>
    {
        type Output = Vector<Float>;

        fn add(self, other: Vector<Float>) -> Vector<Float>
        {
            Vector::<Float> {
                x: self.x + other.x,
                y: self.y + other.y,
                z: self.z + other.z,
            }
        }
    }

    impl<Float: Sub<Output=Float>> Sub for Vector<Float>
    {
        type Output = Vector<Float>;

        fn sub(self, other: Vector<Float>) -> Vector<Float>
        {
            Vector::<Float> {
                x: self.x - other.x,
                y: self.y - other.y,
                z: self.z - other.z,
            }
        }
    }
}
