use std::{cell::LazyCell, time::Instant};

use bit_set::BitSet;
use delegate::delegate;
use glam::{Mat4, Quat, Vec3};

use crate::move_controller::MoveControl;

pub const Z_MAPPING_MAT: LazyCell<Mat4> = LazyCell::new(|| {
    glam::camera::lh::proj::directx::orthographic(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0)
});

pub struct Camera {
    pub move_control: MoveControl,
    pub projection: Mat4,
}

impl Camera {
    delegate! {
        to self.move_control {
            pub fn reset(&mut self);
            pub fn trans_dir(&self, pressed_keys: &BitSet) -> Vec3;
        }
    }

    // pub fn update(&mut self, pressed_keys: &BitSet) -> Mat4 {
    //     self.projection.mul_mat4(
    //         &Z_MAPPING_MAT.mul_mat4(
    //         &self.move_control.update(pressed_keys)
    //     ))
    // }

    pub fn new(mut move_control: MoveControl) -> Self {
        move_control.angular_velocity -= 1.0;
        move_control.trans_velocity -= 1.0;
        Self {
            move_control: move_control,
            projection: glam::camera::lh::proj::directx::perspective_infinite(80.0, 1.0, 0.1)
                .mul_mat4(&Z_MAPPING_MAT),
        }
    }

    pub fn update(&mut self, pressed_keys: &BitSet) -> Mat4 {
        let now = Instant::now();
        let dt = now - self.move_control.last_update;
        self.move_control.last_update = now;
        let dt_sec = dt.as_secs_f32();

        let rot_axis = self.move_control.rotation_axis(pressed_keys);
        let trans_vec = self.move_control.trans_dir(pressed_keys);

        let rot_mat = if rot_axis != Vec3::ZERO {
            self.move_control.rotation =
                Quat::from_axis_angle(rot_axis, self.move_control.angular_velocity * dt_sec)
                    .mul_quat(self.move_control.rotation);
            Mat4::from_quat(self.move_control.rotation)
        } else {
            Mat4::from_quat(self.move_control.rotation)
        };

        //TODO is this accurate - wont this move faster diagonally (no if normalised?)
        let trans_dt = if trans_vec == Vec3::ZERO {
            Mat4::IDENTITY
        } else {
            //rot_axis_mat.mul_mat4(&Mat4::from_translation(trans_vec * self.trans_velocity * dt_sec))
            Mat4::from_translation(
                self.move_control.rotation * trans_vec * self.move_control.trans_velocity * dt_sec,
            )
        };

        //log::info!("trans dvec max: {}",self.trans_velocity * dt_sec);
        //log::info!("trans vec: {}",(trans_vec * self.trans_velocity) * dt_sec);
        //log::info!("trans_dt: {}",trans_dt);

        self.move_control.translation = self.move_control.translation.mul_mat4(&trans_dt);

        //log::info!("translation: {}",self.translation);

        self.projection * rot_mat * self.move_control.translation
    }
}
