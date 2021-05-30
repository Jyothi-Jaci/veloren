use super::{
    super::{vek::*, Animation},
    BipedLargeSkeleton, SkeletonAttr,
};
use common::{
    comp::item::{AbilitySpec, ToolKind},
    states::utils::StageSection,
};
use std::f32::consts::PI;

pub struct BeamAnimation;

impl Animation for BeamAnimation {
    type Dependency<'a> = (
        (Option<ToolKind>, Option<&'a AbilitySpec>),
        (Option<ToolKind>, Option<&'a AbilitySpec>),
        f32,
        Vec3<f32>,
        Option<StageSection>,
        f32,
        f32,
    );
    type Skeleton = BipedLargeSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"biped_large_beam\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "biped_large_beam")]
    #[allow(clippy::single_match)] // TODO: Pending review in #587
    fn update_skeleton_inner<'a>(
        skeleton: &Self::Skeleton,
        (
            (active_tool_kind, active_tool_spec),
            _second_tool_kind,
            global_time,
            velocity,
            stage_section,
            acc_vel,
            timer,
        ): Self::Dependency<'a>,
        anim_time: f32,
        rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;
        let mut next = (*skeleton).clone();

        let speed = Vec2::<f32>::from(velocity).magnitude();

        let lab: f32 = 0.65 * s_a.tempo;
        let speednorm = (speed / 12.0).powf(0.4);
        let foothoril = (acc_vel * lab + PI * 1.45).sin() * speednorm;
        let foothorir = (acc_vel * lab + PI * (0.45)).sin() * speednorm;
        let footrotl = ((1.0 / (0.5 + (0.5) * ((acc_vel * lab + PI * 1.4).sin()).powi(2))).sqrt())
            * ((acc_vel * lab + PI * 1.4).sin())
            * speednorm;

        let footrotr = ((1.0 / (0.5 + (0.5) * ((acc_vel * lab + PI * 0.4).sin()).powi(2))).sqrt())
            * ((acc_vel * lab + PI * 0.4).sin())
            * speednorm;
        let subtract = global_time - timer;
        let check = subtract - subtract.trunc();
        let mirror = (check - 0.5).signum();
        next.jaw.position = Vec3::new(0.0, s_a.jaw.0, s_a.jaw.1);
        next.jaw.orientation = Quaternion::rotation_x(0.0);

        next.main.position = Vec3::new(0.0, 0.0, 0.0);
        next.main.orientation = Quaternion::rotation_x(0.0);

        next.hand_l.position = Vec3::new(s_a.grip.1, 0.0, s_a.grip.0);
        next.hand_r.position = Vec3::new(-s_a.grip.1, 0.0, s_a.grip.0);

        next.hand_l.orientation = Quaternion::rotation_x(0.0);
        next.hand_r.orientation = Quaternion::rotation_x(0.0);
        let (move1base, move2shake, _move2base, move3) = match stage_section {
            Some(StageSection::Buildup) => (
                (anim_time.powf(0.25)).min(1.0),
                (anim_time * 15.0 + PI).sin(),
                (anim_time * 10.0 + PI).sin(),
                0.0,
            ),
            Some(StageSection::Cast) => (
                1.0,
                (anim_time * 15.0 + PI).sin(),
                anim_time.powf(0.25),
                0.0,
            ),
            Some(StageSection::Recover) => (1.0, 1.0, 1.0, anim_time),
            _ => (0.0, 0.0, 0.0, 0.0),
        };
        let pullback = 1.0 - move3;
        let move1 = move1base * pullback;
        match active_tool_kind {
            Some(ToolKind::Sceptre) | Some(ToolKind::Staff) => {
                next.control_l.position = Vec3::new(-1.0, 3.0, 12.0);
                next.control_r.position =
                    Vec3::new(1.0 + move1 * 5.0, 2.0 + move1 * 1.0, 2.0 + move1 * 14.0);

                next.control.position = Vec3::new(
                    -3.0 + move1 * -5.0,
                    3.0 + s_a.grip.0 / 1.2 + move1 * 3.0 + move2shake * 1.0,
                    -11.0 + -s_a.grip.0 / 2.0 + move1 * -2.0,
                );
                next.head.orientation =
                    Quaternion::rotation_x(move1 * -0.2) * Quaternion::rotation_y(move1 * 0.2);
                next.jaw.orientation = Quaternion::rotation_x(0.0);

                next.control_l.orientation =
                    Quaternion::rotation_x(PI / 2.0) * Quaternion::rotation_y(-0.5);
                next.control_r.orientation = Quaternion::rotation_x(PI / 2.5 + move1 * 0.4)
                    * Quaternion::rotation_y(0.5)
                    * Quaternion::rotation_z(move1 * 1.2 + move2shake * 0.5);

                next.control.orientation = Quaternion::rotation_x(-0.2 + move1 * -0.1)
                    * Quaternion::rotation_y(-0.1 + move1 * 0.6);
                next.shoulder_l.position = Vec3::new(
                    -s_a.shoulder.0,
                    s_a.shoulder.1,
                    s_a.shoulder.2 - foothorir * 1.0,
                );
                next.shoulder_l.orientation =
                    Quaternion::rotation_x(move1 * 0.2 + 0.3 + 0.8 * speednorm + (footrotr * -0.2));
                next.shoulder_r.position = Vec3::new(
                    s_a.shoulder.0,
                    s_a.shoulder.1,
                    s_a.shoulder.2 - foothoril * 1.0,
                );
                next.shoulder_r.orientation =
                    Quaternion::rotation_x(move1 * 0.2 + 0.3 + 0.6 * speednorm + (footrotl * -0.2));
                next.torso.orientation = Quaternion::rotation_x(move1 * -0.1);
                next.torso.position = Vec3::new(0.0, 0.0, move1 * 1.0);
            },
            Some(ToolKind::Natural) => {
                if let Some(AbilitySpec::Custom(spec)) = active_tool_spec {
                    match spec.as_str() {
                        "Tidal Warrior" => {
                            if mirror > 0.0 {
                                next.head.orientation = Quaternion::rotation_z(move1 * -0.6);
                                next.upper_torso.orientation = Quaternion::rotation_z(move1 * 0.6);
                                next.lower_torso.orientation = Quaternion::rotation_z(move1 * -0.6);

                                next.shoulder_l.orientation = Quaternion::rotation_z(move1 * 0.3);
                                next.hand_l.position = Vec3::new(-14.0 + move1 * 3.0, 2.0, -4.0);

                                next.hand_l.orientation =
                                    Quaternion::rotation_x(PI / 3.0 + move2shake * -0.07)
                                        * Quaternion::rotation_y(move1 * -0.5)
                                        * Quaternion::rotation_z(-0.35 + move2shake * 0.07);
                                next.hand_r.position = Vec3::new(14.0 + move1 - 3.0, 2.0, -4.0);

                                next.hand_r.orientation =
                                    Quaternion::rotation_x(PI / 3.0 + move2shake * 0.07)
                                        * Quaternion::rotation_y(move1 * -0.5)
                                        * Quaternion::rotation_z(0.35 - move2shake * 0.07);

                                next.shoulder_r.orientation = Quaternion::rotation_z(move1 * -0.3);
                            } else {
                                next.head.orientation = Quaternion::rotation_z(move1 * 0.6);
                                next.upper_torso.orientation = Quaternion::rotation_z(move1 * -0.6);
                                next.lower_torso.orientation = Quaternion::rotation_z(move1 * 0.6);

                                next.shoulder_l.orientation = Quaternion::rotation_z(move1 * -0.3);
                                next.hand_l.position = Vec3::new(-14.0 + move1 * 3.0, 2.0, -4.0);

                                next.hand_l.orientation =
                                    Quaternion::rotation_x(PI / 3.0 + move2shake * 0.07)
                                        * Quaternion::rotation_y(move1 * 0.5)
                                        * Quaternion::rotation_z(-0.35 + move2shake * 0.07);
                                next.hand_r.position = Vec3::new(14.0 + move1 - 3.0, 2.0, -4.0);

                                next.hand_r.orientation =
                                    Quaternion::rotation_x(PI / 3.0 + move2shake * -0.07)
                                        * Quaternion::rotation_y(move1 * 0.5)
                                        * Quaternion::rotation_z(0.35 - move2shake * -0.07);

                                next.shoulder_r.orientation = Quaternion::rotation_z(move1 * 0.3);
                            };
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }

        next
    }
}
