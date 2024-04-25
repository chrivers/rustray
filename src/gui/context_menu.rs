use cgmath::Deg;

use crate::{
    gui::IconButton,
    light::{AreaLight, Attenuation, DirectionalLight, PointLight, SpotLight},
    scene::BoxScene,
    types::{Color, Float, Vector, Vectorx},
};

pub fn add_light<F>(ui: &mut egui::Ui, scene: &mut BoxScene<F>) -> bool
where
    F: Float,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    let mut res = false;

    macro_rules! add_light_option {
        ($name:ident, $code:block) => {
            if ui
                .icon_button($name::<F>::ICON, stringify!($name))
                .clicked()
            {
                scene.add_light($code);
                res = true;
            }
        };
    }

    let attn = Attenuation {
        a: F::ZERO,
        b: F::ONE,
        c: F::ZERO,
    };

    add_light_option!(PointLight, {
        PointLight::new(Vector::ZERO, attn, Color::WHITE)
    });

    add_light_option!(DirectionalLight, {
        DirectionalLight::new(-Vector::UNIT_Z, Color::WHITE)
    });

    add_light_option!(SpotLight, {
        SpotLight {
            attn,
            umbra: Deg(F::from_u32(45)).into(),
            penumbra: Deg(F::from_u32(60)).into(),
            pos: Vector::ZERO,
            dir: -Vector::UNIT_Z,
            color: Color::WHITE,
        }
    });

    add_light_option!(AreaLight, {
        AreaLight::new(
            attn,
            Vector::ZERO,
            -Vector::UNIT_Z,
            Vector::UNIT_Y,
            Color::WHITE,
            F::from_u32(10),
            F::from_u32(10),
        )
    });

    res
}
