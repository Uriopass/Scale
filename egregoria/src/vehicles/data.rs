use crate::engine_interaction::Selectable;
use crate::map_dynamic::{Itinerary, ParkingManagement};
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject};
use crate::rendering::assets::{AssetID, AssetRender};
use crate::utils::rand_provider::RandProvider;
use crate::utils::rand_world;
use crate::Egregoria;
use common::{GameInstant, GameTime, Z_CAR};
use geom::Color;
use geom::{Spline, Transform, Vec2};
use imgui_inspect::InspectDragf;
use imgui_inspect_derive::*;
use legion::Entity;
use map_model::{Map, ParkingSpotID};
use serde::{Deserialize, Serialize};

/// The duration for the parking animation.
pub const TIME_TO_PARK: f32 = 4.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VehicleID(pub Entity);

debug_inspect_impl!(VehicleID);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VehicleState {
    Parked(ParkingSpotID),
    Driving,
    /// Panicked when it notices it's in a gridlock
    Panicking(GameInstant),
    RoadToPark(Spline, f32, ParkingSpotID),
}

debug_inspect_impl!(VehicleState);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VehicleKind {
    Car,
    Truck,
    Bus,
}

#[derive(Clone, Debug, Serialize, Deserialize, Inspect)]
pub struct Vehicle {
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,

    pub state: VehicleState,
    pub kind: VehicleKind,

    /// Used to detect gridlock
    pub flag: u64,
}

#[must_use]
pub fn put_vehicle_in_coworld(goria: &mut Egregoria, w: f32, trans: Transform) -> Collider {
    Collider(goria.write::<CollisionWorld>().insert(
        trans.position(),
        PhysicsObject {
            dir: trans.direction(),
            radius: w * 0.5,
            group: PhysicsGroup::Vehicles,
            ..Default::default()
        },
    ))
}

impl VehicleKind {
    pub fn width(self) -> f32 {
        match self {
            VehicleKind::Car => 4.5,
            VehicleKind::Truck => 6.0,
            VehicleKind::Bus => 9.0,
        }
    }

    pub fn acceleration(self) -> f32 {
        match self {
            VehicleKind::Car => 3.0,
            VehicleKind::Truck => 2.5,
            VehicleKind::Bus => 2.0,
        }
    }

    pub fn deceleration(self) -> f32 {
        match self {
            VehicleKind::Car | VehicleKind::Bus | VehicleKind::Truck => 9.0,
        }
    }

    pub fn min_turning_radius(self) -> f32 {
        match self {
            VehicleKind::Car => 3.0,
            VehicleKind::Truck => 4.0,
            VehicleKind::Bus => 5.0,
        }
    }

    pub fn cruising_speed(self) -> f32 {
        match self {
            VehicleKind::Car => 12.0,
            VehicleKind::Truck | VehicleKind::Bus => 10.0,
        }
    }

    pub fn ang_acc(self) -> f32 {
        match self {
            VehicleKind::Car => 1.0,
            VehicleKind::Truck => 0.9,
            VehicleKind::Bus => 0.8,
        }
    }
}

pub fn spawn_parked_vehicle(
    goria: &mut Egregoria,
    kind: VehicleKind,
    near: Vec2,
) -> Option<VehicleID> {
    let r: f64 = rand_world(goria);

    let map = goria.read::<Map>();

    let time = goria.read::<GameTime>().timestamp;
    let it = Itinerary::wait_until(time + r * 5.0);

    let pm = goria.read::<ParkingManagement>();

    let spot_id = pm.reserve_near(near, &map)?;

    let pos = map.parking.get(spot_id).unwrap().trans; // Unwrap ok: Gotten using reserve_near

    drop(map);
    drop(pm);

    Some(VehicleID(make_vehicle_entity(
        goria,
        pos,
        Vehicle::new(kind, spot_id),
        it,
        false,
    )))
}

pub fn make_vehicle_entity(
    goria: &mut Egregoria,
    trans: Transform,
    vehicle: Vehicle,
    it: Itinerary,
    mk_collider: bool,
) -> Entity {
    let asset_id = match vehicle.kind {
        VehicleKind::Car => AssetID::CAR,
        VehicleKind::Truck => AssetID::TRUCK,
        VehicleKind::Bus => unreachable!(),
    };

    let tint = match vehicle.kind {
        VehicleKind::Car => get_random_car_color(&mut *goria.write::<RandProvider>()),
        _ => Color::WHITE,
    };

    let w = vehicle.kind.width();
    let e = goria.world.push((
        AssetRender {
            id: asset_id,
            hide: false,
            scale: w,
            tint,
            z: Z_CAR,
        },
        trans,
        Kinematics::from_mass(1000.0),
        Selectable::default(),
        vehicle,
        it,
    ));

    if mk_collider {
        let c = put_vehicle_in_coworld(goria, w, trans);
        goria.world.entry(e).unwrap().add_component(c);
    }

    e
}

pub fn get_random_car_color(r: &mut RandProvider) -> Color {
    let car_colors: [(Color, f32); 9] = [
        (Color::from_hex(0x22_22_22), 0.22),  // Black
        (Color::from_hex(0xff_ff_ff), 0.19),  // White
        (Color::from_hex(0x66_66_66), 0.17),  // Gray
        (Color::from_hex(0xb8_b8_b8), 0.14),  // Silver
        (Color::from_hex(0x1a_3c_70), 0.1),   // Blue
        (Color::from_hex(0xd8_22_00), 0.1),   // Red
        (Color::from_hex(0x7c_4b_24), 0.02),  // Brown
        (Color::from_hex(0xd4_c6_78), 0.015), // Gold
        (Color::from_hex(0x72_cb_19), 0.015), // Green
    ];

    let total: f32 = car_colors.iter().map(|x| x.1).sum();

    let r = r.random::<f32>() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}

impl Vehicle {
    pub fn new(kind: VehicleKind, spot: ParkingSpotID) -> Vehicle {
        Self {
            ang_velocity: 0.0,
            wait_time: 0.0,
            state: VehicleState::Parked(spot),
            kind,
            flag: 0,
        }
    }
}

debug_inspect_impl!(VehicleKind);
