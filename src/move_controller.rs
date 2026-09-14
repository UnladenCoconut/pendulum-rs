use core::f32;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use bit_set::BitSet;
use glam::{Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;

///ugh idk if we really should combine these into one move control or not
/// Not only do we have local/world translation variants
/// //same with rotation, current rotation is applied to delta rotation for camera but not object
/// we might also want a variant with gimbal lock on rotation for e.g. a First-Person Camera

#[derive(Clone)]
pub struct MoveControlKeySet {
    pub trans_x_dec: KeyCode,
    pub trans_x_inc: KeyCode,
    pub trans_y_dec: KeyCode,
    pub trans_y_inc: KeyCode,
    pub trans_z_dec: KeyCode,
    pub trans_z_inc: KeyCode,

    pub rot_x_ccw: KeyCode,
    pub rot_x_cw: KeyCode,
    pub rot_y_ccw: KeyCode,
    pub rot_y_cw: KeyCode,
    pub rot_z_ccw: KeyCode,
    pub rot_z_cw: KeyCode,
}

/// computes the matrix for translation and rotation control based
/// off bound keys
#[derive(Clone)]
pub struct MoveControl {
    pub keybinds: MoveControlKeySet,
    pub trans_velocity: f32,
    pub angular_velocity: f32,

    pub rotation: Quat,
    pub translation: Mat4,
    //rotation axis and translational velocity vector are computed at each timestep
    pub last_update: Instant,
}

impl MoveControl {
    pub fn new(keybinds: MoveControlKeySet, trans_velocity: f32, angular_velocity: f32) -> Self {
        Self {
            keybinds: keybinds,
            trans_velocity: trans_velocity,
            angular_velocity: angular_velocity,

            rotation: Quat::IDENTITY,
            translation: Mat4::IDENTITY,

            last_update: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.translation = Mat4::IDENTITY;
        self.rotation = Quat::IDENTITY;
    }

    pub fn rotation_axis(&self, pressed_keys: &BitSet) -> Vec3 {
        let mut rot_axis = Vec3::ZERO;
        if pressed_keys.contains(self.keybinds.rot_x_ccw as usize) {
            rot_axis.x += 1.0;
        }
        if pressed_keys.contains(self.keybinds.rot_x_cw as usize) {
            rot_axis.x -= 1.0;
        }
        if pressed_keys.contains(self.keybinds.rot_y_ccw as usize) {
            rot_axis.y += 1.0;
        }
        if pressed_keys.contains(self.keybinds.rot_y_cw as usize) {
            rot_axis.y -= 1.0;
        }
        if pressed_keys.contains(self.keybinds.rot_z_ccw as usize) {
            rot_axis.z -= 1.0;
        }
        if pressed_keys.contains(self.keybinds.rot_z_cw as usize) {
            rot_axis.z += 1.0;
        }
        rot_axis = rot_axis.normalize_or_zero();
        rot_axis
    }

    pub fn trans_dir(&self, pressed_keys: &BitSet) -> Vec3 {
        let mut trans_vec = Vec3::ZERO;

        if pressed_keys.contains(self.keybinds.trans_x_inc as usize) {
            trans_vec.x += 1.0;
        }
        if pressed_keys.contains(self.keybinds.trans_x_dec as usize) {
            trans_vec.x -= 1.0;
        }
        if pressed_keys.contains(self.keybinds.trans_y_inc as usize) {
            trans_vec.y += 1.0;
        }
        if pressed_keys.contains(self.keybinds.trans_y_dec as usize) {
            trans_vec.y -= 1.0;
        }
        if pressed_keys.contains(self.keybinds.trans_z_inc as usize) {
            trans_vec.z += 1.0;
        }
        if pressed_keys.contains(self.keybinds.trans_z_dec as usize) {
            trans_vec.z -= 1.0;
        }
        trans_vec = trans_vec.normalize_or_zero();
        trans_vec
    }

    /// # Returns:
    /// The updated matrix transform for the object
    pub fn update(&mut self, pressed_keys: &BitSet) -> Mat4 {
        let now = Instant::now();
        let dt = now - self.last_update;
        self.last_update = now;
        let dt_sec = dt.as_secs_f32();

        let rot_axis = self.rotation_axis(pressed_keys);
        let trans_vec = self.trans_dir(pressed_keys);

        let rot_mat = if rot_axis != Vec3::ZERO {
            self.rotation = self.rotation.mul_quat(Quat::from_axis_angle(
                rot_axis,
                self.angular_velocity * dt_sec,
            ));
            Mat4::from_quat(self.rotation)
        } else {
            Mat4::from_quat(self.rotation)
        };

        //TODO is this accurate - wont this move faster diagonally (no if normalised?)
        let trans_dt = if trans_vec == Vec3::ZERO {
            Mat4::IDENTITY
        } else {
            Mat4::from_translation(trans_vec * self.trans_velocity * dt_sec)
            //Mat4::from_translation(self.rotation * trans_vec * self.trans_velocity * dt_sec)
        };

        //log::info!("trans dvec max: {}",self.trans_velocity * dt_sec);
        //log::info!("trans vec: {}",(trans_vec * self.trans_velocity) * dt_sec);
        //log::info!("trans_dt: {}",trans_dt);

        self.translation = self.translation.mul_mat4(&trans_dt);

        //log::info!("translation: {}",self.translation);

        rot_mat * self.translation
    }
}
