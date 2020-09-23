use egregoria::rendering::{Color, LinearColor};
use egregoria::utils::Restrict;
use geom::vec2;
use map_model::{
    BuildingKind, Lane, LaneKind, Map, ProjectKind, TrafficBehavior, TurnKind, CROSSWALK_WIDTH,
};
use std::ops::Mul;
use wgpu_engine::{
    compile_shader, CompiledShader, FrameContext, GfxContext, InstanceRaw, Mesh, ShadedBatch,
    ShadedBatchBuilder, ShadedInstanceRaw, Shaders, SpriteBatch, SpriteBatchBuilder, Tesselator,
};

#[derive(Clone, Copy)]
struct Crosswalk;

impl Shaders for Crosswalk {
    fn vert_shader() -> CompiledShader {
        compile_shader("assets/shaders/crosswalk.vert", None)
    }

    fn frag_shader() -> CompiledShader {
        compile_shader("assets/shaders/crosswalk.frag", None)
    }
}

pub struct RoadRenderer {
    map_mesh: Option<Mesh>,
    arrows: Option<SpriteBatch>,
    arrow_builder: SpriteBatchBuilder,
    crosswalks: Option<ShadedBatch<Crosswalk>>,
}

const Z_LOT: f32 = 0.2;
const Z_WALKWAY: f32 = 0.205;
const Z_INTER_BG: f32 = 0.208;
const Z_LANE_BG: f32 = 0.21;
const Z_LANE: f32 = 0.22;
const Z_SIDEWALK: f32 = 0.23;
const Z_ARROW: f32 = 0.24;
const Z_CROSSWALK: f32 = 0.25;
const Z_HOUSE: f32 = 0.28;
const Z_SIGNAL: f32 = 0.29;

impl RoadRenderer {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let arrow_builder = SpriteBatchBuilder::from_path(gfx, "assets/arrow_one_way.png");

        gfx.register_pipeline::<ShadedBatch<Crosswalk>>();

        RoadRenderer {
            map_mesh: None,
            arrows: None,
            arrow_builder,
            crosswalks: None,
        }
    }

    fn map_mesh(&self, map: &Map, mut tess: Tesselator, gfx: &GfxContext) -> Option<Mesh> {
        let lo_gray: LinearColor = Color::gray(0.2).into();
        let mi_gray: LinearColor = Color::gray(0.25).into();
        let hi_gray: LinearColor = Color::gray(0.42).into();

        let inters = map.intersections();
        let lanes = map.lanes();

        let gray_line = LinearColor::gray(0.3);

        for l in lanes.values() {
            tess.set_color(gray_line);

            let or_src = l.orientation_from(l.src);
            let or_dst = -l.orientation_from(l.dst);

            let w = l.width + 0.5;
            tess.draw_polyline_with_dir(l.points.as_slice(), or_src, or_dst, Z_LANE_BG, w);

            tess.set_color(match l.kind {
                LaneKind::Walking => hi_gray,
                LaneKind::Parking => lo_gray,
                _ => mi_gray,
            });
            let z = match l.kind {
                LaneKind::Walking => Z_SIDEWALK,
                _ => Z_LANE,
            };

            tess.draw_polyline_with_dir(l.points.as_slice(), or_src, or_dst, z, l.width - 0.5);
        }

        let mut p = Vec::with_capacity(8);
        for inter in inters.values() {
            if inter.roads.is_empty() {
                tess.set_color(gray_line);
                tess.draw_circle(inter.pos, Z_LANE_BG, 5.5);

                tess.set_color(mi_gray);
                tess.draw_circle(inter.pos, Z_LANE, 5.0);
                continue;
            }

            tess.set_color(mi_gray);
            tess.draw_filled_polygon(inter.polygon.as_slice(), Z_INTER_BG);

            for turn in inter
                .turns()
                .iter()
                .filter(|turn| matches!(turn.kind, TurnKind::WalkingCorner))
            {
                tess.set_color(gray_line);
                let id = turn.id;

                let w = lanes[id.src].width;

                let first_dir = -lanes[id.src].orientation_from(id.parent);
                let last_dir = lanes[id.dst].orientation_from(id.parent);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, Z_LANE_BG, w + 0.5);

                tess.set_color(hi_gray);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                let z = Z_SIDEWALK;

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, z, w - 0.5);
            }
        }

        for building in map.buildings().values() {
            tess.set_color(Color::gray(0.3));
            tess.draw_filled_polygon(building.walkway.as_slice(), Z_WALKWAY);

            let col = match building.kind {
                BuildingKind::House => Color::new(0.5, 0.52, 0.5, 1.0),
                BuildingKind::Workplace => Color::new(0.48, 0.48, 0.5, 1.0),
            };
            tess.set_color(col);
            tess.draw_filled_polygon(building.exterior.as_slice(), Z_HOUSE);
        }

        for lot in map.lots().values() {
            tess.set_color(Color::new(0.2, 0.6, 0.25, 1.0));
            tess.draw_filled_polygon(&lot.shape.corners, Z_LOT);
        }
        tess.meshbuilder.build(gfx)
    }

    fn render_lane_signals(n: &Lane, sr: &mut Tesselator, time: u64) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + dir_perp * 2.5 + dir * 2.5;

        if n.control.is_stop_sign() {
            sr.set_color(LinearColor::WHITE);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.5, 8, std::f32::consts::FRAC_PI_8);

            sr.set_color(LinearColor::RED);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.4, 8, std::f32::consts::FRAC_PI_8);
            return;
        }

        let size = 0.5; // light size

        sr.color = Color::gray(0.3).into();
        sr.draw_rect_cos_sin(r_center, Z_SIGNAL, size + 0.1, size * 3.0 + 0.1, dir);

        for i in -1..2 {
            sr.draw_circle(r_center + i as f32 * dir_perp * size, Z_SIGNAL, size * 0.5);
        }
        sr.set_color(match n.control.get_behavior(time) {
            TrafficBehavior::RED | TrafficBehavior::STOP => LinearColor::RED,
            TrafficBehavior::ORANGE => LinearColor::ORANGE,
            TrafficBehavior::GREEN => LinearColor::GREEN,
        });

        let offset = match n.control.get_behavior(time) {
            TrafficBehavior::RED => -size,
            TrafficBehavior::ORANGE => 0.0,
            TrafficBehavior::GREEN => size,
            _ => unreachable!(),
        };

        sr.draw_circle(r_center + offset * dir_perp, Z_SIGNAL, size * 0.5);
    }

    fn signals_render(map: &Map, time: u64, sr: &mut Tesselator) {
        match sr.cull_rect {
            Some(rect) => {
                if rect.w.max(rect.h) > 1500.0 {
                    return;
                }
                for n in map
                    .spatial_map()
                    .query_rect(rect)
                    .filter_map(|k| match k {
                        ProjectKind::Road(id) => Some(id),
                        _ => None,
                    })
                    .flat_map(|id| map.roads()[id].lanes_iter())
                    .map(|(id, _)| &map.lanes()[id])
                {
                    Self::render_lane_signals(n, sr, time);
                }
            }
            None => {
                for n in map.lanes().values() {
                    Self::render_lane_signals(n, sr, time);
                }
            }
        }
    }

    fn arrows(&mut self, map: &Map, gfx: &GfxContext) -> Option<SpriteBatch> {
        self.arrow_builder.instances.clear();
        let lanes = map.lanes();
        for road in map.roads().values() {
            let fade = (road.length - 5.0 - road.src_interface - road.dst_interface)
                .mul(0.2)
                .restrict(0.0, 1.0);

            let r_lanes = road.lanes_iter().filter(|(_, kind)| kind.vehicles());
            let n_arrows = ((road.length / 50.0) as i32).max(1);

            for (id, _) in r_lanes {
                let lane = &lanes[id];
                let l = lane.points.length();
                for i in 0..n_arrows {
                    let (mid, dir) = lane
                        .points
                        .point_dir_along(l * (1.0 + i as f32) / (1.0 + n_arrows as f32));

                    self.arrow_builder.instances.push(InstanceRaw::new(
                        mid,
                        dir,
                        Z_ARROW,
                        [0.3 + fade * 0.1; 3],
                        4.0,
                    ));
                }
            }
        }
        self.arrow_builder.build(gfx)
    }

    fn crosswalks(&mut self, map: &Map, gfx: &GfxContext) -> Option<ShadedBatch<Crosswalk>> {
        let mut builder = ShadedBatchBuilder::<Crosswalk>::new();

        let lanes = map.lanes();
        for (inter_id, inter) in map.intersections() {
            for turn in inter.turns() {
                let id = turn.id;

                if matches!(turn.kind, TurnKind::Crosswalk) {
                    let from = lanes[id.src].get_inter_node_pos(inter_id);
                    let to = lanes[id.dst].get_inter_node_pos(inter_id);

                    let l = (to - from).magnitude();

                    if l < 4.0 {
                        continue;
                    }

                    let dir = (to - from) / l;
                    let pos = from + dir * 2.25;
                    let height = l - 4.5;

                    builder.instances.push(ShadedInstanceRaw::new(
                        pos,
                        Z_CROSSWALK,
                        dir,
                        vec2(height, CROSSWALK_WIDTH),
                        LinearColor::WHITE.into(),
                    ));
                }
            }
        }
        builder.build(&gfx)
    }

    pub fn render(
        &mut self,
        map: &mut Map,
        time: u64,
        tess: &mut Tesselator,
        ctx: &mut FrameContext,
    ) {
        if map.dirty {
            self.map_mesh = self.map_mesh(map, Tesselator::new(None, 15.0), &ctx.gfx);
            self.arrows = self.arrows(map, &ctx.gfx);
            self.crosswalks = self.crosswalks(map, &ctx.gfx);

            map.dirty = false;
        }

        if let Some(x) = self.map_mesh.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.arrows.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.crosswalks.clone() {
            ctx.draw(x);
        }

        Self::signals_render(map, time, tess);
    }
}